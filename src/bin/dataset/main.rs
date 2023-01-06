use common::config::{try_load_config, DEFAULT_CONFIG};

use crate::market::MAX_KLINES;

mod market;

fn main() {
    let config = try_load_config(DEFAULT_CONFIG);
    let file_path =
        "/home/quinten/Programming/Rust/ml-crypto-trading-bot/datasets/BTC-Hourly-Binance.csv";
    let symbol = "BTC/USDT";

    let mut writer = csv::WriterBuilder::new()
        .delimiter(b',')
        .from_path(file_path)
        .unwrap();

    writer
        .write_record(&[
            "open_time",
            "close_time",
            "symbol",
            "open",
            "high",
            "low",
            "close",
            "volume",
            "quote_asset_volume",
        ])
        .expect("failed to write header");

    let mut i: u64 = 0;

    for kline in market::get_binance_klines(config.binance, symbol.into()) {
        let count = i % MAX_KLINES as u64;

        if count == 0 && i != 0 {
            println!();
        }

        print!("\rwriting record {} / {}", count + 1, MAX_KLINES);

        writer
            .write_record(&[
                kline.open_time.to_string(),
                kline.close_time.to_string(),
                symbol.to_string(),
                kline.open.to_string(),
                kline.high.to_string(),
                kline.low.to_string(),
                kline.close.to_string(),
                kline.volume.to_string(),
                kline.quote_asset_volume.to_string(),
            ])
            .expect("failed to write record");

        i += 1;
    }
}
