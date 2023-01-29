use crate::{config::AppConfig, dataset::DataSet, market::Market, model::Model};
use std::{
    sync::{atomic::AtomicBool, mpsc::Sender, Arc},
    time::Instant,
};

pub trait Strategy {
    fn execute(&self, running: Arc<AtomicBool>, tx: &Sender<String>);
}

pub struct LightGBMStrategy<M: Market> {
    pub config: AppConfig,
    pub model: Model,
    pub market: M,
}

impl<M: Market> LightGBMStrategy<M> {
    pub fn new(config: AppConfig, market: M) -> Self {
        Self {
            config,
            market,
            model: Model::new(),
        }
    }

    pub fn train_model(&self, dataset: DataSet) -> anyhow::Result<lightgbm::Booster> {
        // Train the model.
        info!("Training model");
        let start = Instant::now();
        let booster = self.model.train(dataset)?;
        let end = Instant::now();
        let elapsed = end.duration_since(start);
        info!("Model trained successfully! Time elapsed: {:?}", elapsed);
        Ok(booster)
    }
}
