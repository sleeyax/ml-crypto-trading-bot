# :robot: ml-crypto-trading-bot

Cryptocurrency trading bot using Machine Learning and Rust. Inspired by [CyberPunkMetalHead/cryptocurrency-machine-learning-prediction-algo-trading](https://github.com/CyberPunkMetalHead/cryptocurrency-machine-learning-prediction-algo-trading).

## :warning: Disclaimer

This bot has been developed in response to another project and out of curiousity to see if I could build an improved version in Rust. The efficiency of the strategy itself hasn't been thoroughly tested. Use this program at your own risk!

## :book: Strategy

The trading strategy is relatively simple:

- Fetch the last X days of hourly kline (candle) data from Binance.
- Train a machine learning model on the data. I'm using [LightGBM](https://lightgbm.readthedocs.io/en/v3.3.2/), which is a fast gradient boosting framework that uses tree based learning algorithms. It's not perfect and predictions aren't nearly as close to reality as other solutions like recurrent neural networks (RNN) like LSTM, but in my testing it can provide a good indicator for basic market movements (price up or down), which is all I need for this strategy.
- Using the trained model, predict the current candle `high` price. If it's lower than the current `open` or `close` (i.e current) price, wait for the next candle and start over. Otherwise, place a buy order.
- Finally, the bot waits for the price to go up until the the prediction is reached. If the prediction isn't reached by the end of the candle, it just waits until the prediction is reached eventually.

## :info: Installation & usage

Install [Rust](https://www.rust-lang.org/tools/install) and clone this repository:

```bash
$ git clone https://github.com/sleeyax/ml-crypto-trading-bot.git
$ cd ml-crypto-trading-bot
```

Then, copy the config file and edit it accordingly (should be self explanatory):

```bash
$ cp config.example.yaml config.yaml
$ vim config.yaml # or use any other text editor of choice to edit the config file
```

To run the bot in development mode, execute:

```bash
$ RUST_LOG=debug cargo run
```

To run the bot in production mode, execute:

```bash
$ RUST_LOG=info cargo run
```

You can also build a binary release with `cargo build -r` and copy it + your config file to a VPS or raspberry pi.
