use crate::config::{try_load_config, DEFAULT_CONFIG};
use crate::dataset::DataSet;
use crate::market::{BinanceKlineInterval, BinanceKlineOptions, BinanceMarket};
use crate::model::Model;

pub mod config;
pub mod dataset;
pub mod market;
pub mod model;
pub mod utils;

fn main() {
    let config = try_load_config(DEFAULT_CONFIG);
    let symbol = "BTC/USDT";

    let binance_market = BinanceMarket::new(config.binance);
    let model = Model::new();

    let dataset = DataSet::from_binance(
        &binance_market,
        BinanceKlineOptions {
            pair: symbol.into(),
            interval: BinanceKlineInterval::Hourly,
            limit: Some(7 * 24), // last 7 days (as hours)
            start: None,
            end: None,
        },
    );

    let booster = model.train(dataset.clone()).unwrap();

    let current_price = binance_market
        .get_price(symbol)
        .expect("failed to get current price");

    let result = booster
        .predict(vec![
            vec![current_price.price], // features (open)
        ])
        .unwrap();

    let score = result[0][0];

    // println!("{:?}", dataset);
    println!(
        "last open, high in dataset: {}, {}",
        dataset.0.last().unwrap()[0],
        dataset.1.last().unwrap()
    );
    println!("current price: {}", current_price.price);
    println!("predicted high: {}", score);

    // TODO: place trade orders :)
}
