use std::{future::Future, time::Duration};

use binance::{
    api::Binance,
    market::Market,
    model::{KlineSummaries, KlineSummary},
};
use common::config::BinanceConfig;
use genawaiter::rc::{Co, Gen};

pub const MAX_KLINES: u16 = 1500;

/// Generator that returns ALL hourly klines up untill now.
/// Note that the returned results are unsorted.
pub fn get_binance_klines(
    config: BinanceConfig,
    pair: String,
) -> genawaiter::rc::Gen<KlineSummary, (), impl Future<Output = ()>> {
    Gen::new(|co: Co<KlineSummary>| async move {
        let market: Market = Binance::new(Some(config.api_key), Some(config.api_secret));
        let mut end_time: Option<u64> = None;

        loop {
            match market.get_klines(
                pair.replace("/", ""),
                "1h",
                Some(MAX_KLINES),
                None,
                end_time,
            ) {
                Ok(klines) => match klines {
                    KlineSummaries::AllKlineSummaries(klines) => {
                        if klines.len() == 0 {
                            break;
                        }

                        let first_kline = klines.first().unwrap().clone();

                        for kline in klines {
                            co.yield_(kline).await;
                        }

                        end_time = Some(
                            first_kline.close_time as u64
                                - Duration::from_secs(3600).as_millis() as u64,
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
