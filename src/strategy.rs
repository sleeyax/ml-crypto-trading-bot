use anyhow::anyhow;
use binance::websockets::{WebSockets, WebsocketEvent};

use crate::{
    binance_market::{BinanceKlineInterval, BinanceKlineOptions, BinanceMarket},
    config::AppConfig,
    dataset::DataSet,
    market::Market,
    model::Model,
    utils::{calculate_profit, to_symbol},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub trait Strategy {
    fn execute(&self, running: Arc<AtomicBool>);
}

pub struct LightGBMStrategy<M: Market> {
    config: AppConfig,
    model: Model,
    market: M,
}

impl<M: Market> LightGBMStrategy<M> {
    pub fn new(config: AppConfig, market: M) -> Self {
        Self {
            config,
            market,
            model: Model::new(),
        }
    }

    pub fn train_model(&self, dataset: DataSet) -> anyhow::Result<lightgbm::Booster> {
        // Train the model.
        info!("Training model");
        let start = Instant::now();
        let booster = self.model.train(dataset)?;
        let end = Instant::now();
        let elapsed = end.duration_since(start);
        info!("Model trained successfully! Time elapsed: {:?}", elapsed);
        Ok(booster)
    }
}

impl LightGBMStrategy<BinanceMarket> {
    /// Load dataset data (features, labels) from binance klines API.
    fn load_dataset(&self) -> DataSet {
        DataSet::from_binance(
            &self.market,
            BinanceKlineOptions {
                pair: self.config.symbol.clone(),
                interval: BinanceKlineInterval::Hourly,
                limit: Some(7 * 24), // last 7 days (as hours)
                start: None,
                end: None,
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
            let prediction = booster.predict(vec![vec![current_kline_close]]).unwrap();
            let score = prediction[0][0];

            info!(
                "Current kline open, close: {}, {}.",
                current_kline_open, current_kline_close
            );
            info!("Predicted high: {}.", score);

            // Wait for the right moment tot place a trade.
            if score < current_kline_open || score < current_kline_close {
                let minutes = 10;
                warn!("Predicted value {} is lower than the open ({}) or current ({}) price, skipping trade and waiting {} minutes until the next prediction.", score, current_kline_open, current_kline_close, minutes);
                thread::sleep(Duration::from_secs(60 * minutes));
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

            // Wait for price to go up.
            // We don't wait for the prediction to match exactly.
            // The prediction should only serve as a general indicator (up or down).
            // Instead we'll wait and sell only once a specific profit percentage has been reached.
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
                        let (profit, profit_percentage) = calculate_profit(
                            self.config.trade.amount,
                            initial_price,
                            selling_price,
                        );

                        debug!(
                              "Current profit: {} (%{}). Candle open: {}, close {}, high: {}, low: {}",
                              profit,
                              profit_percentage,
                              kline_event.kline.open,
                              kline_event.kline.close,
                              kline_event.kline.low,
                              kline_event.kline.high
                          );

                        if profit_percentage >= self.config.trade.profit_percentage {
                            info!(
                                "Placing sell order for an estimated profit of {} USD.",
                                profit,
                            );
                            self.market
                                .place_sell_order(
                                    &self.config.symbol,
                                    self.config.trade.amount,
                                    self.config.trade.test,
                                )
                                .expect("failed to place sell order");
                            info!(
                                "Successfully sold for an estimated profit of {} USD.",
                                profit
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