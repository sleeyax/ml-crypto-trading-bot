use std::{future::Future, time::Duration};

use binance::{
    api::Binance,
    market::Market,
    model::{KlineSummaries, KlineSummary},
};
use common::config::BinanceConfig;
use genawaiter::rc::{Co, Gen};

/// The maximum amount of Klines binance supports in the response body.
pub const BINANCE_MAX_KLINES: u16 = 1500;

/// The starting timestamp when binance started indexing market data.
/// Human readable date: `2017-08-17T04:00:00.000Z`.
const BINANCE_MARKET_EPOCH: u64 = 1502942400000;

pub struct BinanceMarket {
    /// Config struct encapsulating Binance API keys.
    config: BinanceConfig,
}

#[allow(dead_code)]
pub enum BinanceKlineInterval {
    Hourly,
    Daily,
}

impl ToString for BinanceKlineInterval {
    fn to_string(&self) -> String {
        String::from(match self {
            BinanceKlineInterval::Hourly => "1h",
            BinanceKlineInterval::Daily => "1d",
        })
    }
}

impl BinanceKlineInterval {
    fn to_seconds(&self) -> u64 {
        match self {
            BinanceKlineInterval::Hourly => 3600,
            BinanceKlineInterval::Daily => 86400,
        }
    }
}

pub struct BinanceKlineOptions {
    /// The target trading pair (symbol).
    /// May optionally contain one forward slash (`/`).
    /// Examples: `BTC/USDT` ,`BTCEUR`.
    pub pair: String,

    /// Candle open/close intervals.
    /// Only supports hourly and daily intervals for now.
    pub interval: BinanceKlineInterval,

    /// Maximum amount of results to return.
    /// Defaults to `BINANCE_MAX_KLINES`.
    pub limit: Option<u16>,

    /// Start time.
    /// Defaults to `BINANCE_MARKET_EPOCH`.
    pub start: Option<u64>,

    /// End time.
    /// Defaults to `None`.
    pub end: Option<u64>,
}

impl BinanceMarket {
    pub fn new(config: BinanceConfig) -> Self {
        BinanceMarket { config }
    }

    /// Generator that returns ALL klines from `BINANCE_MARKET_EPOCH` until now.
    pub fn get_klines(
        &self,
        options: BinanceKlineOptions,
    ) -> genawaiter::rc::Gen<KlineSummary, (), impl Future<Output = ()>> {
        let market: Market = Binance::new(
            Some(self.config.api_key.clone()),
            Some(self.config.api_secret.clone()),
        );

        Gen::new(|co: Co<KlineSummary>| async move {
            let mut start_time: Option<u64> = options.start.or(Some(BINANCE_MARKET_EPOCH));

            loop {
                match market.get_klines(
                    options.pair.replace("/", ""),
                    options.interval.to_string(),
                    options.limit.or(Some(BINANCE_MAX_KLINES)),
                    start_time,
                    options.end,
                ) {
                    Ok(klines) => match klines {
                        KlineSummaries::AllKlineSummaries(klines) => {
                            if klines.len() == 0 {
                                break;
                            }

                            let cursor = klines.last().unwrap().clone();

                            for kline in klines {
                                co.yield_(kline).await;
                            }

                            start_time = Some(
                                cursor.close_time as u64
                                    + Duration::from_secs(options.interval.to_seconds()).as_millis()
                                        as u64,
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
}
