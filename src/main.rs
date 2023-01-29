use crate::binance_market::BinanceMarket;
use crate::config::TelegramConfig;
use crate::config::{try_load_config, DEFAULT_CONFIG};
use crate::strategy::Strategy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread;
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

async fn start_telegram_bot(config: TelegramConfig, notification_rx: Receiver<String>) {
    info!("Starting telegram bot.");

    let telegram_bot = Bot::new(config.bot_token);
    let tb = telegram_bot.clone();

    let handle_listener = tokio::spawn(async {
        teloxide::repl(tb, |bot: Bot, msg: Message| async move {
            // TODO: allow user to query current status and/or logs of the trading bot?
            bot.send_message(
                msg.chat.id,
                format!(
                    "Sorry, I don't support any commands yet. Chat ID: {}",
                    msg.chat.id
                ),
            )
            .await?;
            Ok(())
        })
        .await;
    });

    let handle_sender = tokio::spawn(async move {
        let msg = notification_rx.recv().unwrap();

        telegram_bot
            .send_message(config.chat_id.to_string(), msg)
            .await
            .unwrap();
    });

    let _ = tokio::join!(handle_listener);

    handle_sender.abort(); // TODO: find better way to stop sending messages on quit.
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let (notification_tx, notification_rx) = channel::<String>();

    let config = try_load_config(DEFAULT_CONFIG);
    let running = Arc::new(AtomicBool::new(true));
    let telegram_config = config.telegram.clone();

    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("Exiting program.");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let handle_trading_bot = thread::spawn(move || {
        if config.trade.test {
            warn!("Bot is running in test mode. No real funds will be spent.");
        } else {
            warn!("Bot is running in production mode. Real funds will be spent!");
        }

        let market = BinanceMarket::new(config.binance.clone());
        let strategy = LightGBMStrategy::new(config, market);
        strategy.execute(running.clone(), &notification_tx);
    });

    start_telegram_bot(telegram_config, notification_rx).await;

    handle_trading_bot
        .join()
        .expect("Failed to join the trading bot thread.");
}
