use lightgbm::Error;
use serde_json::{json, Value};

use crate::dataset::DataSet;

pub struct Model {
    params: Value,
}

impl Model {
    pub fn new() -> Self {
        let params = json! {
            {
                "objective": "regression",
                // "num_iterations": 30442,
                // "num_leaves": 13,
                "num_leaves": 20,
                "learning_rate": 0.05,
                "n_estimators": 720,
            }
        };

        Model { params }
    }

    /// Retrain the model with the given dataset.
    pub fn train(&self, DataSet(features, labels): DataSet) -> Result<lightgbm::Booster, Error> {
        let train_dataset = lightgbm::Dataset::from_mat(features, labels).unwrap();
        lightgbm::Booster::train(train_dataset, &self.params)
    }
}
