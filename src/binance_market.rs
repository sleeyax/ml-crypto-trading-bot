use crate::{config::BinanceConfig, market::Market};
use binance::{
    account::Account,
    api::Binance as BinanceApi,
    market::Market as Market_,
    model::{KlineSummaries, KlineSummary},
};
use genawaiter::rc::{Co, Gen};
use std::{future::Future, time::Duration};

/// The maximum amount of Klines binance supports in the response body.
pub const BINANCE_MAX_KLINES: u16 = 1500;

/// The starting timestamp when binance started indexing market data.
/// Human readable date: `2017-08-17T04:00:00.000Z`.
pub const BINANCE_MARKET_EPOCH: u64 = 1502942400000;

pub struct BinanceMarket {
    market: Market_,
    account: Account,
}

pub struct BinanceKlineOptions {
    /// The target trading pair/symbol.
    pub pair: String,

    /// Candle open/close intervals.
    /// Only supports hourly and daily intervals for now.
    pub interval: BinanceKlineInterval,

    /// Maximum amount of results to return.
    /// Defaults to `BINANCE_MAX_KLINES`.
    pub limit: Option<u16>,

    /// Start time.
    /// Set to `BINANCE_MARKET_EPOCH` to specify the very beginning.
    /// Defaults to `None`.
    pub start: Option<u64>,

    /// End time.
    /// Defaults to `None`.
    pub end: Option<u64>,
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

impl BinanceMarket {
    pub fn new(config: BinanceConfig) -> Self {
        let market: Market_ = BinanceApi::new(
            Some(config.api_key.clone()),
            Some(config.api_secret.clone()),
        );
        let account: Account = Account::new(
            Some(config.api_key.clone()),
            Some(config.api_secret.clone()),
        );
        BinanceMarket { market, account }
    }

    /// Generator that returns klines from binance.
    /// Defaults to ALL klines from `BINANCE_MARKET_EPOCH` until now.
    pub fn get_klines(
        &self,
        options: BinanceKlineOptions,
    ) -> genawaiter::rc::Gen<KlineSummary, (), impl Future<Output = ()>> {
        let market = self.market.clone();

        Gen::new(|co: Co<KlineSummary>| async move {
            let mut start_time: Option<u64> = options.start;

            loop {
                match market.get_klines(
                    &options.pair,
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

impl Market for BinanceMarket {
    fn get_price(&self, symbol: &str) -> anyhow::Result<f64> {
        let price_symbol = self.market.get_price(symbol).map_err(map_binance_error)?;
        Ok(price_symbol.price)
    }

    fn place_buy_order(&self, symbol: &str, quantity: f64, test: bool) -> anyhow::Result<()> {
        if test {
            self.account
                .test_market_buy_using_quote_quantity(symbol, quantity)
                .map_err(map_binance_error)
        } else {
            self.account
                .market_buy_using_quote_quantity(symbol, quantity)
                .map(|_| ())
                .map_err(map_binance_error)
        }
    }

    fn place_sell_order(&self, symbol: &str, quantity: f64, test: bool) -> anyhow::Result<()> {
        if test {
            self.account
                .test_market_sell_using_quote_quantity(symbol, quantity)
                .map_err(map_binance_error)
        } else {
            self.account
                .market_sell_using_quote_quantity(symbol, quantity)
                .map(|_| ())
                .map_err(map_binance_error)
        }
    }
}

/// Converts a binance error to an anyhow error.
fn map_binance_error(err: binance::errors::Error) -> anyhow::Error {
    anyhow::anyhow!(err.to_string())
}
