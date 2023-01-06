use config::{Config, File};
use serde::Deserialize;

pub const DEFAULT_CONFIG: &str = "config.yaml";

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub binance: BinanceConfig,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct BinanceConfig {
    pub api_key: String,
    pub api_secret: String,
}

/// Tries to load the specified config file.
/// Panics when it can't be found or has an invalid format.
pub fn load_config(name: &str) -> AppConfig {
    let settings = Config::builder()
        .add_source(File::with_name(name))
        .build()
        .expect("failed to load config file");

    settings
        .try_deserialize::<AppConfig>()
        .expect("invalid config file format")
}
