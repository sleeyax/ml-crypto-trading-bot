# ml-crypto-trading-bot
Cryptocurrency trading bot using Machine Learning and Rust. Initially inspired by [CyberPunkMetalHead/cryptocurrency-machine-learning-prediction-algo-trading](https://github.com/CyberPunkMetalHead/cryptocurrency-machine-learning-prediction-algo-trading).

## Strategy
The trading strategy is relatively simple:
- Fetch the last X days of hourly kline (candle) data from Binance.
- Train a machine learning model on the data. I'm using [LightGBM](https://lightgbm.readthedocs.io/en/v3.3.2/), which is a fast gradient boosting framework that uses tree based learning algorithms. It's not perfect and predictions aren't nearly as close to reality as other solutions like recurrent neural networks (RNN) like LSTM, but in my testing it can provide a good indicator for basic market movements (price up or down), which is all I need for this strategy.
-  Using the trained model, predict the current candle `high` price. If it's lower than the current candle `open` or `close` (i.e current) price, wait 10 minutes and start over. Otherwise, place a buy order. The reason I decided to wait 10 minutes instead of the whole lifetime of the candle is that I can't be sure when the bot was started. For example, when I start the bot at the very start of a new candle, the price could still drasticially change depending on current market conditions. Thus, setting an interval of 10 minutes allows for more opportunities.
- Finally, the bot waits for the price to go up until the configured profit is reached. Once reached, a sell order is placed and the bot starts over. The bot never sells at a loss; it keeps waiting forever until a profit is reached in order to sell.
