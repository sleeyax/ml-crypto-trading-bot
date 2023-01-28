use csv::ReaderBuilder;

use crate::binance_market::{BinanceKlineOptions, BinanceMarket};

pub type Features = Vec<Vec<f64>>;
pub type Labels = Vec<f32>;

#[derive(Debug, Clone)]
pub struct DataSet(pub Features, pub Labels);

impl From<&str> for DataSet {
    /// Extracts the necessary `labels` and `features` from the given dataset.
    /// Only supports CSV files.
    fn from(csv_file_path: &str) -> Self {
        let reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b',')
            .from_path(csv_file_path);

        let mut labels: Labels = Vec::new();
        let mut features: Features = Vec::new();

        for result in reader.unwrap().records() {
            let record = result.unwrap();

            let label = record[4].parse::<f32>().unwrap(); // high
            let feature: Vec<f64> = vec![record[3].parse::<f64>().unwrap()]; // open

            labels.push(label);
            features.push(feature);
        }

        Self(features, labels)
    }
}

impl DataSet {
    /// Extracts the necessary `labels` and `features` from kline data from Binance.
    pub fn from_binance(market: &BinanceMarket, options: BinanceKlineOptions) -> Self {
        let mut labels: Labels = Vec::new();
        let mut features: Features = Vec::new();

        for kline in market.get_klines(options) {
            let label = kline.high.parse::<f32>().unwrap();
            let feature: Vec<f64> = vec![kline.open.parse::<f64>().unwrap()];
            labels.push(label);
            features.push(feature);
        }

        Self(features, labels)
    }
}
