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
                "num_leaves": 13,
                "num_iterations": 1000,
                "bagging_fraction": 0.6065339345698,
                "feature_fraction": 0.99999999,
                "lambda_l1": 0.0120496605030283,
                "lambda_l2": 0.139677140815755,
                "max_bin_by_feature": 1033,
                "verbose": -1
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
