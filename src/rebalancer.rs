use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;
use reqwest::Client as ReqwestClient;

pub struct Rebalancer {
    ooga_booga_endpoint: String,
    client: ReqwestClient,
    threshold: f64,
}

impl Rebalancer {
    pub fn new(ooga_booga_endpoint: String, threshold: f64) -> Self {
        Self {
            ooga_booga_endpoint,
            client: ReqwestClient::builder().build().unwrap(),
            threshold,
        }
    }

    pub async fn check_rebalance_needed(&self, current_price: f64, target_price: f64) -> bool {
        let price_deviation = (current_price - target_price).abs() / target_price;
        price_deviation > self.threshold
    }

    pub async fn rebalance_position(&self, position_id: U256) -> Result<()> {
        // TODO: Implement rebalancing through Ooga Booga
        // 1. Call Ooga Booga API to get optimal new position parameters
        // 2. Remove old liquidity
        // 3. Provide new liquidity at optimal range
        Ok(())
    }
} 