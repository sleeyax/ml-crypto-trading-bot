use crate::binance_market::BinanceMarket;
use crate::config::{try_load_config, DEFAULT_CONFIG};
use crate::strategy::Strategy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use strategy::LightGBMStrategy;
use teloxide::requests::Requester;
use teloxide::types::Message;
use teloxide::Bot;

#[macro_use]
extern crate log;

pub mod binance_market;
pub mod binance_strategy;
pub mod config;
pub mod dataset;
pub mod market;
pub mod model;
pub mod strategy;
pub mod utils;

async fn start_telegram_bot(token: String, _running: Arc<AtomicBool>) {
    println!("Starting telegram bot in separate thread.");
    let telegram_bot = Bot::new(token);
    teloxide::repl(telegram_bot, |bot: Bot, msg: Message| async move {
        bot.send_message(msg.chat.id, "hello").await?;
        Ok(())
    })
    .await;
}

fn main() {
    env_logger::init();

    let config = try_load_config(DEFAULT_CONFIG);

    if config.trade.test {
        warn!("Bot is running in test mode. No real funds will be spent.");
    } else {
        warn!("Bot is running in production mode. Real funds will be spent!");
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("Exiting program.");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let market = BinanceMarket::new(config.binance.clone());
    let strategy = LightGBMStrategy::new(config, market);
    strategy.execute(running.clone());

    // TODO: finish telegram bot integration

    // tokio::runtime::Runtime::new().unwrap().block_on(async {
    //     join!(
    //         start_telegram_bot(config.telegram.bot_token.clone(), running.clone()),
    //         start_trading_bot(config, running.clone())
    //     );
    // });
}
