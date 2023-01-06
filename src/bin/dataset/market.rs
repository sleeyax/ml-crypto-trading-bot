use std::{future::Future, time::Duration};

use binance::{
    api::Binance,
    market::Market,
    model::{KlineSummaries, KlineSummary},
};
use common::config::BinanceConfig;
use genawaiter::rc::{Co, Gen};

/// The maximum amount of Klines in the response binance supports.
pub const MAX_KLINES: u16 = 1500;

/// The starting timestamp when binance started indexing market data.
/// Human readable date: `2017-08-17T04:00:00.000Z`.
const BINANCE_EPOCH: u64 = 1502942400000;

/// Generator that returns ALL hourly klines from `2017-08-17T04:00:00.000Z` until now.
pub fn get_binance_klines(
    config: BinanceConfig,
    pair: String,
) -> genawaiter::rc::Gen<KlineSummary, (), impl Future<Output = ()>> {
    Gen::new(|co: Co<KlineSummary>| async move {
        let market: Market = Binance::new(Some(config.api_key), Some(config.api_secret));
        let mut start_time: Option<u64> = Some(BINANCE_EPOCH);

        loop {
            match market.get_klines(
                pair.replace("/", ""),
                "1h",
                Some(MAX_KLINES),
                start_time,
                None,
            ) {
                Ok(klines) => match klines {
                    KlineSummaries::AllKlineSummaries(klines) => {
                        if klines.len() == 0 {
                            break;
                        }

                        let last_kline = klines.last().unwrap().clone();

                        for kline in klines {
                            co.yield_(kline).await;
                        }

                        start_time = Some(
                            last_kline.close_time as u64
                                + Duration::from_secs(3600).as_millis() as u64,
                        );
                    }
                },
                Err(err) => {
                    println!("error during retrieval of historical klines: {}", err);
                    break;
                }
            };
        }
    })
}
