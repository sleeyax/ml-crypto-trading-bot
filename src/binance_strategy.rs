use crate::{
    binance_market::{to_symbol, BinanceKlineInterval, BinanceKlineOptions, BinanceMarket},
    dataset::DataSet,
    market::Market,
    strategy::{LightGBMStrategy, Strategy},
    utils::{calculate_profit, ceil_hour, earlier_seconds, floor_hour, now},
};
use anyhow::anyhow;
use binance::websockets::{WebSockets, WebsocketEvent};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

impl LightGBMStrategy<BinanceMarket> {
    /// Load dataset data (features, labels) from binance klines API.
    fn load_dataset(&self) -> DataSet {
        let one_hour_ago = earlier_seconds(floor_hour(now()), 3600);

        DataSet::from_binance(
            &self.market,
            BinanceKlineOptions {
                pair: self.config.symbol.clone(),
                interval: BinanceKlineInterval::Hourly,
                limit: Some(self.config.binance.dataset_max_days * 24), // last x days as hours
                start: None,
                end: Some(one_hour_ago.as_millis() as u64), // exclude the curent candle
            },
        )
    }

    /// Get the current candle's `open` and `close` price.
    fn get_current_candle(&self) -> anyhow::Result<(f64, f64)> {
        let kline = self
            .market
            .get_klines(BinanceKlineOptions {
                pair: self.config.symbol.clone(),
                interval: BinanceKlineInterval::Hourly,
                limit: Some(1),
                start: None,
                end: None,
            })
            .into_iter()
            .last()
            .ok_or(anyhow!("failed to get current kline"))?;
        let kline_open = kline.open.parse::<f64>().unwrap();
        let kline_close = kline.close.parse::<f64>().unwrap();
        Ok((kline_open, kline_close))
    }
}

impl Strategy for LightGBMStrategy<BinanceMarket> {
    fn execute(&self, running: Arc<AtomicBool>) {
        while running.load(Ordering::SeqCst) {
            let dataset = self.load_dataset();

            info!(
                "Last open, high in dataset: {}, {}",
                dataset.0.last().unwrap()[0],
                dataset.1.last().unwrap()
            );
            // println!("{:?}", dataset);

            // Train the model using latest data from binance.
            let booster = self.train_model(dataset).unwrap();

            // Get the current price candle.
            let (current_kline_open, current_kline_close) = self.get_current_candle().unwrap();

            // Predict the next `high` price.
            let prediction = booster.predict(vec![vec![current_kline_open]]).unwrap();
            let score = prediction[0][0];

            info!(
                "Current kline open, close: {}, {}.",
                current_kline_open, current_kline_close
            );
            info!("Predicted high: {}.", score);

            // Wait until the next candle if the trade is not profitable according to our prediction.
            if score < current_kline_open || score < current_kline_close {
                let duration = ceil_hour(now());
                warn!("Predicted value {} is lower than the open ({}) or current ({}) price, skipping trade and waiting {:?} until the start of the next candle.", score, current_kline_open, current_kline_close, duration);
                thread::sleep(duration);
                continue;
            }

            // Place buy order
            info!(
                "Buying {} {}.",
                self.config.trade.amount,
                self.config.symbol.clone(),
            );
            self.market
                .place_buy_order(
                    &self.config.symbol,
                    self.config.trade.amount,
                    self.config.trade.test,
                )
                .expect("failed to place buy order");
            info!(
                "Bought {} {}.",
                self.config.trade.amount,
                self.config.symbol.clone(),
            );

            // Wait and sell once the prediction has been reached.
            // If the prediction hasn't been reached at the end of the candle, we wait until we can sell the amount at the same price or higher,
            // so we never sell at a loss!
            let mut invalid_prediction_warning_shown = false;
            let start_of_next_candle = ceil_hour(now());
            let connected = AtomicBool::new(true);
            let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
                // Disconnect if we got the signal to terminate the program (e.g. CTRL + C).
                if running.load(Ordering::SeqCst) == false {
                    connected.store(false, Ordering::SeqCst);
                    return Ok(());
                }

                match event {
                    WebsocketEvent::Kline(kline_event) => {
                        let initial_price = current_kline_close;
                        let selling_price = kline_event.kline.close.parse::<f64>().unwrap();

                        debug!(
                            "Candle open: {}, close {}, high: {}, low: {}.",
                            kline_event.kline.open,
                            kline_event.kline.close,
                            kline_event.kline.low,
                            kline_event.kline.high,
                        );
                        debug!(
                            "Initial price: {}, selling price: {} ({} difference).",
                            initial_price,
                            selling_price,
                            selling_price - initial_price
                        );

                        let now = now();
                        let is_predicted = selling_price >= score;
                        let is_end_of_candle = now >= start_of_next_candle;

                        if !is_predicted && is_end_of_candle && !invalid_prediction_warning_shown {
                            warn!("Invalid prediction. End of candle reached. Predicted high was {}, actual high is {}.", score,  kline_event.kline.high.parse::<f64>().unwrap());
                            invalid_prediction_warning_shown = true;
                        }

                        if is_predicted || (is_end_of_candle && selling_price >= initial_price) {
                            let (profit, profit_percentage) = calculate_profit(
                                self.config.trade.amount,
                                initial_price,
                                selling_price,
                            );

                            info!(
                                "Selling {} {} for an estimated profit of {} USD ({}%).",
                                self.config.trade.amount,
                                self.config.symbol.clone(),
                                profit,
                                profit_percentage,
                            );
                            self.market
                                .place_sell_order(
                                    &self.config.symbol,
                                    self.config.trade.amount,
                                    self.config.trade.test,
                                )
                                .expect("failed to place sell order");
                            info!(
                                "Sold {} {} for an estimated profit of {} USD ({}%).",
                                self.config.trade.amount,
                                self.config.symbol.clone(),
                                profit,
                                profit_percentage
                            );

                            connected.store(false, Ordering::SeqCst);
                        }
                    }
                    _ => (),
                };

                Ok(())
            });
            web_socket
                .connect(&format!(
                    "{}@kline_1h",
                    to_symbol(&self.config.symbol).to_lowercase()
                ))
                .expect("websocket failed to connect");
            web_socket.event_loop(&connected).unwrap();
            web_socket.disconnect().unwrap();
        }
    }
}
