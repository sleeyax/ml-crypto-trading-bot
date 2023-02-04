use anyhow::Ok;
use config::{Config, File};
use serde::Deserialize;

pub const DEFAULT_CONFIG: &str = "config.yaml";

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub binance: BinanceConfig,
    pub trade: TradeConfig,
    pub telegram: TelegramConfig,
    pub symbol: String,
    pub verbose: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct BinanceConfig {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct TradeConfig {
    pub amount: f64,
    pub test: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: u64,
}

/// Load the specified config file.
pub fn load_config(name: &str) -> anyhow::Result<AppConfig> {
    let settings = Config::builder()
        .add_source(File::with_name(name))
        .build()?;

    let config = settings.try_deserialize::<AppConfig>()?;

    Ok(config)
}

/// Tries to load the specified config file.
/// Panics when it can't be found or has an invalid format.
pub fn try_load_config(name: &str) -> AppConfig {
    load_config(name).expect("failed to load config or invalid format")
}

#[cfg(test)]
mod tests {
    use crate::config::try_load_config;

    use super::load_config;

    #[test]
    fn test_load_config() {
        let config = load_config("doesntexist.yaml");
        assert_eq!(config.is_err(), true);
        assert_eq!(config.unwrap_err().to_string().contains("not found"), true);
        let config = load_config("config.example.yaml");
        assert_eq!(config.is_err(), false);
    }

    #[test]
    fn test_try_load_config() {
        let config = try_load_config("config.example.yaml");
        assert_eq!(config.symbol, "BTCUSDT");
        assert_eq!(config.verbose, true);
        assert_eq!(config.binance.api_key, "paste your binance api key here");
        assert_eq!(
            config.binance.api_secret,
            "paste your binance api secret here"
        );
        assert_eq!(config.trade.test, true);
        assert_eq!(config.trade.amount, 50.0);
        assert_eq!(config.telegram.bot_token, "123456789:blablabla");
    }
}
