use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub pool_address: String,
    pub token_a: String,
    pub token_b: String,
    pub tick_spacing: i32,
    pub num_ticks: u32,
    pub rebalance_threshold: f64,
    pub ooga_booga_endpoint: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pool_address: String::new(),
            token_a: String::new(),
            token_b: String::new(),
            tick_spacing: 60, // Default tick spacing for Bulla pools
            num_ticks: 10,     // Number of ticks to provide liquidity to
            rebalance_threshold: 0.05, // 5% price deviation threshold
            ooga_booga_endpoint: String::from("https://api.oogabooga.com"),
        }
    }
} 