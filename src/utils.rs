use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    binance_market::{
        BinanceKlineInterval, BinanceKlineOptions, BinanceMarket, BINANCE_MARKET_EPOCH,
        BINANCE_MAX_KLINES,
    },
    config::{try_load_config, DEFAULT_CONFIG},
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

pub fn calculate_profit(investment: f64, initial_price: f64, selling_price: f64) -> (f64, f64) {
    let price = investment * (selling_price / initial_price) - investment;
    let percentage = (price / investment) * 100.0;
    (price, percentage)
}

/// Returns the primary duration minus the secondary duration.
pub fn earlier(duration: Duration, duration_earlier: Duration) -> Duration {
    let time = UNIX_EPOCH + duration;
    let earlier = time - duration_earlier;
    earlier.duration_since(UNIX_EPOCH).unwrap()
}

/// Returns the primary duration minus the secondary duration in seconds.
pub fn earlier_seconds(duration: Duration, secs: u64) -> Duration {
    earlier(duration, Duration::from_secs(secs))
}

/// Returns the nearest hour, rounded down.
///
/// Example:
///
/// ```
/// let timestamp = Duration::from(1674940162060); // 2023-01-28T21:09:22.060Z
/// let floor = floor_hour(timestamp);             // 2023-01-28T21:00:00.0Z
/// ```
pub fn floor_hour(timestamp: Duration) -> Duration {
    let system_time = UNIX_EPOCH + timestamp;

    // Get the number of seconds since the nearest hour.
    let seconds_since_nearest_hour =
        system_time.duration_since(UNIX_EPOCH).unwrap().as_secs() % 3600;

    // Subtract the number of seconds since the nearest hour from the current timestamp to get the nearest hour.
    let nearest_hour = timestamp.as_millis() as u64 / 1000 - seconds_since_nearest_hour;

    Duration::from_secs(nearest_hour)
}

/// Returns the nearest hour, rounded up.
///
/// Example:
///
/// ```
/// let timestamp = Duration::from(1674940162060); // 2023-01-28T21:09:22.060Z
/// let ceil = ceil_hour(timestamp);               // 2023-01-28T22:00:00.0Z
/// ```
pub fn ceil_hour(timestamp: Duration) -> Duration {
    let system_time = UNIX_EPOCH + timestamp;

    // Get the number of seconds since the nearest hour.
    let seconds_since_nearest_hour =
        system_time.duration_since(UNIX_EPOCH).unwrap().as_secs() % 3600;

    // Add the number of seconds until the next hour to the current timestamp to get the upper hour.
    let upper_hour = timestamp.as_millis() as u64 / 1000 + (3600 - seconds_since_nearest_hour);

    Duration::from_secs(upper_hour)
}

/// Returns the current time as a unix epoch timestamp encapsulated in a `Duration`.
/// Use `as_millis()` to acess the value accordingly.
pub fn now() -> Duration {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).unwrap()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::utils::{calculate_profit, ceil_hour, floor_hour};

    use super::earlier;

    #[test]
    fn test_calculate_profit() {
        assert_eq!(calculate_profit(10.0, 20.0, 30.0), (5.0, 50.0));
        assert_eq!(
            calculate_profit(30.0, 20878.0, 20900.0),
            (0.03161222339304359, 0.10537407797681198)
        );
    }

    #[test]
    fn test_earlier() {
        let timestamp = 1674925200000; // 5 PM UTC
        let expected = 1674921600000; // 4 PM UTC
        let hour_earlier = earlier(Duration::from_millis(timestamp), Duration::from_secs(3600));
        assert_eq!(hour_earlier.as_millis(), expected);
    }

    #[test]
    fn test_floor_hour() {
        let upper_timestamp = 1674928799000; // 2023-01-28T17:59:59 UTC
        let lower_timestamp = 1674925201000; // 2023-01-28T17:00:01Z UTC
        let expected_timestamp = 1674925200000; // 2023-01-28T17:00:00Z UTC
        assert_eq!(
            floor_hour(Duration::from_millis(upper_timestamp)).as_millis(),
            expected_timestamp
        );
        assert_eq!(
            floor_hour(Duration::from_millis(lower_timestamp)).as_millis(),
            expected_timestamp
        );
        assert_eq!(
            floor_hour(Duration::from_millis(expected_timestamp as u64)).as_millis(),
            expected_timestamp
        );
    }

    #[test]
    fn test_ceil_hour() {
        let upper_timestamp = 1674928799000; // 2023-01-28T17:59:59 UTC
        let lower_timestamp = 1674925201000; // 2023-01-28T17:00:01Z UTC
        let expected_timestamp = 1674928800000; // 2023-01-28T18:00:00Z UTC
        assert_eq!(
            ceil_hour(Duration::from_millis(upper_timestamp)).as_millis(),
            expected_timestamp
        );
        assert_eq!(
            ceil_hour(Duration::from_millis(lower_timestamp)).as_millis(),
            expected_timestamp
        );
    }
}
