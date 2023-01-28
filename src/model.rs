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

#[cfg(test)]
mod tests {
    use crate::dataset::DataSet;

    use super::Model;

    #[test]
    fn test_train() {
        let model = Model::new();
        let dataset = DataSet(vec![vec![0.0]], vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let booster = model.train(dataset).unwrap();
        let prediction = booster.predict(vec![vec![6.0]]).unwrap();
        let score = prediction[0][0];
        assert_eq!(score, 1.0);
    }
}
