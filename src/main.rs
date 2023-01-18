use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{try_load_config, DEFAULT_CONFIG};
use crate::dataset::DataSet;
use crate::market::{BinanceKlineInterval, BinanceKlineOptions, BinanceMarket};
use crate::model::Model;
use binance::websockets::{WebSockets, WebsocketEvent};
use paris::Logger;
use utils::{calculate_profit, to_symbol};

pub mod config;
pub mod dataset;
pub mod market;
pub mod model;
pub mod utils;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("Exiting program.");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let config = try_load_config(DEFAULT_CONFIG);

    let binance_market = BinanceMarket::new(config.binance);
    let model = Model::new();
    let mut log = Logger::new();

    if config.trade.test {
        log.warn("Bot is running in test mode. No real funds will be spent.");
    } else {
        log.warn("Bot is running in production mode. Real funds will be spent!");
    }

    while running.load(Ordering::SeqCst) {
        // Load dataset data (features, labels) from binance klines API.
        let dataset = DataSet::from_binance(
            &binance_market,
            BinanceKlineOptions {
                pair: config.symbol.clone(),
                interval: BinanceKlineInterval::Hourly,
                limit: Some(7 * 24), // last 7 days (as hours)
                start: None,
                end: None,
            },
        );

        // Train the model.
        log.loading("Training model");
        let start = Instant::now();
        let booster = model.train(dataset.clone()).unwrap();
        let end = Instant::now();
        let elapsed = end.duration_since(start);
        log.success(format_args!(
            "Model trained successfully! Time elapsed: {:?}",
            elapsed
        ));

        // Get the current price candle.
        let current_kline = binance_market
            .get_klines(BinanceKlineOptions {
                pair: config.symbol.clone(),
                interval: BinanceKlineInterval::Hourly,
                limit: Some(1),
                start: None,
                end: None,
            })
            .into_iter()
            .last()
            .unwrap();
        let current_kline_open = current_kline.open.parse::<f64>().unwrap();
        let current_kline_close = current_kline.close.parse::<f64>().unwrap();

        // Predict the next `high` price.
        let prediction = booster.predict(vec![vec![current_kline_close]]).unwrap();
        let score = prediction[0][0];

        // println!("{:?}", dataset);
        log.info(format_args!(
            "Last open, high in dataset: {}, {}",
            dataset.0.last().unwrap()[0],
            dataset.1.last().unwrap()
        ));
        log.info(format_args!(
            "Current kline open, close, high: {}, {}, {}.",
            current_kline_open, current_kline_close, current_kline.high
        ));
        log.info(format_args!("Predicted high: {}.", score));

        // Wait for the right moment tot place a trade.
        if score < current_kline_open || score < current_kline_close {
            let minutes = 10;
            log.warn(format_args!("Predicted value {} is lower than the open ({}) or current ({}) price, skipping trade and waiting {} minutes until the next prediction.", score, current_kline_open, current_kline_close, minutes));
            thread::sleep(Duration::from_secs(60 * minutes));
            continue;
        }

        // Place buy order
        log.loading(format_args!(
            "Buying {} {}.",
            config.trade.amount,
            config.symbol.clone(),
        ));
        binance_market
            .place_buy_order(&config.symbol, config.trade.amount, config.trade.test)
            .expect("failed to place buy order");
        log.success(format_args!(
            "Bought {} {}.",
            config.trade.amount,
            config.symbol.clone(),
        ));

        // Wait for price to go up.
        // We don't wait for the prediction to match exactly.
        // The prediction should only serve as a general indicator (up or down).
        // Instead we'll wait and sell only once a specific profit percentage has been reached.
        let connected = AtomicBool::new(true);
        let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
            // Disconnect if we got the signal to terminate the program (CTRL + C).
            if running.load(Ordering::SeqCst) == false {
                connected.store(false, Ordering::SeqCst);
                return Ok(());
            }

            match event {
                WebsocketEvent::Kline(kline_event) => {
                    let initial_price = current_kline_close;
                    let selling_price = kline_event.kline.close.parse::<f64>().unwrap();
                    let (profit, profit_percentage) =
                        calculate_profit(config.trade.amount, initial_price, selling_price);

                    if config.verbose {
                        log.log(format_args!(
                            "Current profit: {} (%{}). Candle open: {}, close {}, high: {}, low: {}",
                            profit,
                            profit_percentage,
                            kline_event.kline.open,
                            kline_event.kline.close,
                            kline_event.kline.low,
                            kline_event.kline.high
                        ));
                    }

                    if profit_percentage >= config.trade.profit_percentage {
                        log.loading(format_args!(
                            "Placing sell order for an estimated profit of {} USD.",
                            profit,
                        ));
                        binance_market
                            .place_sell_order(
                                &config.symbol,
                                config.trade.amount,
                                config.trade.test,
                            )
                            .expect("failed to place sell order");
                        log.success(format_args!(
                            "Successfully sold for an estimated profit of {} USD.",
                            profit
                        ));

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
                to_symbol(&config.symbol).to_lowercase()
            ))
            .expect("websocket failed to connect");
        web_socket.event_loop(&connected).unwrap();
        web_socket.disconnect().unwrap();
    }
}
