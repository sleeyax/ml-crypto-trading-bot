use anyhow::Result;

/// Shared trait to be implemented by all supported markets.
pub trait Market {
    /// Returns the current price of the specified symbol or pair.
    fn get_price(&self, symbol: &str) -> Result<f64>;

    /// Places a buy order on the market.
    fn place_buy_order(&self, symbol: &str, quantity: f64, test: bool) -> anyhow::Result<()>;

    /// Places a sell order on the market.
    fn place_sell_order(&self, symbol: &str, quantity: f64, test: bool) -> anyhow::Result<()>;
}
