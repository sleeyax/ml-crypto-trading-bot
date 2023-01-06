use binance::{api::Binance, market::Market, model::KlineSummary};
use common::config::{load_config, DEFAULT_CONFIG};

fn main() {
  let config = load_config(DEFAULT_CONFIG);
  let market: Market = Binance::new(Some(config.binance.api_key), Some(config.binance.api_secret));
  // last 10 5min klines (candlesticks) for a symbol:
  // TODO: get as much hourly data as possible
  match market.get_klines("BNBETH", "5m", 10, None, None) {
    Ok(klines) => {   
        match klines {
            binance::model::KlineSummaries::AllKlineSummaries(klines) => {
                let kline: KlineSummary = klines[0].clone(); // You need to iterate over the klines
                println!(
                    "Open: {}, High: {}, Low: {}",
                    kline.open, kline.high, kline.low
                )
            }
        }
    },
    Err(e) => println!("Error: {}", e),
}
}
