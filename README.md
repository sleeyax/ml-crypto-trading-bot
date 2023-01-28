# ml-crypto-trading-bot

Cryptocurrency trading bot using Machine Learning and Rust. Initially inspired by [CyberPunkMetalHead's repository](https://github.com/CyberPunkMetalHead/cryptocurrency-machine-learning-prediction-algo-trading).

## Strategy

The trading strategy is relatively simple:

- Fetch the last X days of hourly kline (candle) data from Binance.
- Train a machine learning model on the data. I'm using [LightGBM](https://lightgbm.readthedocs.io/en/v3.3.2/), which is a fast gradient boosting framework that uses tree based learning algorithms. It's not perfect and predictions aren't nearly as close to reality as other solutions like recurrent neural networks (RNN) like LSTM, but in my testing it can provide a good indicator for basic market movements (price up or down), which is all I need for this strategy.
- Using the trained model, predict the current candle `high` price. If it's lower than the current `open` or `close` (i.e current) price, wait for the next candle and start over. Otherwise, place a buy order.
- Finally, the bot waits for the price to go up until the the prediction is reached. If the prediction isn't reached by the end of the candle, it waits and sells the traded amount back for the buying price or higher. Thus, the bot never sells at a loss; it will always wait until some kind of profit is reached.
