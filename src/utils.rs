use crate::{
    config::{try_load_config, DEFAULT_CONFIG},
    market::{
        BinanceKlineInterval, BinanceKlineOptions, BinanceMarket, BINANCE_MARKET_EPOCH,
        BINANCE_MAX_KLINES,
    },
};

/// Fetch ALL klines from Binance and write them to a CSV file.
pub fn save_binance_dataset(file_path: &str, symbol: &str) {
    let config = try_load_config(DEFAULT_CONFIG);

    let binance_market = BinanceMarket::new(config.binance);
    let binance_kline_options = BinanceKlineOptions {
        pair: symbol.into(),
        interval: BinanceKlineInterval::Hourly,
        limit: None,
        start: Some(BINANCE_MARKET_EPOCH),
        end: None,
    };

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

    for kline in binance_market.get_klines(binance_kline_options) {
        let count = i % BINANCE_MAX_KLINES as u64;

        if count == 0 && i != 0 {
            println!();
        }

        print!("\rwriting record {} / {}", count + 1, BINANCE_MAX_KLINES);

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

pub fn to_symbol(symbol: &str) -> String {
    symbol.replace("/", "")
}
