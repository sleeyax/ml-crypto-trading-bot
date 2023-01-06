use lightgbm::{Booster, Dataset};
use serde_json::json;
use std::iter::zip;

pub type Features = Vec<Vec<f64>>;
pub type Labels = Vec<f32>;

/// Extracts the necessary `labels` and `features` from the given dataset.
/// Only supports CSV files.
fn load_dataset(file_path: &str) -> (Features, Labels) {
    let reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_path(file_path);

    let mut labels: Labels = Vec::new();
    let mut features: Features = Vec::new();

    for result in reader.unwrap().records() {
        let record = result.unwrap();

        let label = record[4].parse::<f32>().unwrap(); // high
        let feature: Vec<f64> = vec![record[3].parse::<f64>().unwrap()]; // open

        labels.push(label);
        features.push(feature);
    }

    (features, labels)
}

/// Test the given model for accuracy.
fn test_model(booster: &Booster, test_dataset_path: &str) {
    let fluctuation = 50_f32;

    let (test_features, test_labels) = load_dataset(test_dataset_path);

    let result = booster.predict(test_features).unwrap();
    let result_count = result[0].len();

    let mut total_accurate_predictions = 0;
    for (label, pred) in zip(&test_labels, &result[0]) {
        let diff = (*label - *pred as f32).abs();
        if diff <= fluctuation {
            total_accurate_predictions += 1;
        }
        // println!("label: {}, predicted: {}, diff: {}", label, pred, diff);
    }

    let accuracy_percentage =
        (total_accurate_predictions as f32 / result_count as f32 * 100_f32).round();
    println!(
        "accuracy: {} / {} (~{}%, fluctuation: {})",
        total_accurate_predictions, result_count, accuracy_percentage, fluctuation
    );
}

fn main() {
    let dataset =
        "/home/quinten/Programming/Rust/ml-crypto-trading-bot/datasets/BTC-Hourly-Binance.csv";
    let output_model = "model.trained";

    let (train_features, train_labels) = load_dataset(dataset);

    let train_dataset = Dataset::from_mat(train_features, train_labels).unwrap();

    let params = json! {
        {
            "objective": "regression",
            "num_iterations": 30442,
            "num_leaves": 13,
        }
    };

    let booster = Booster::train(train_dataset, &params).unwrap();
    booster
        .save_file(output_model)
        .expect("failed to save trained model");
    println!("trained model saved to {}", output_model);

    test_model(&booster, dataset);
}
