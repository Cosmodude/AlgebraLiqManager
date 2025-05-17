use anyhow::Result;
use ethers::prelude::*;
use crate::onchain::erc20::ERC20Token;
use crate::onchain::bulla_pool::BullaPool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Pool<P> {
    address: Address,
    token_a: Address,
    token_b: Address,
    token_a_decimals: u8,
    token_b_decimals: u8,
    tick_spacing: i32,
    contract: Option<BullaPool<Arc<Provider<P>>>>,
}

impl<P: JsonRpcClient + 'static> Pool<P> {
    pub async fn new(address: Address, provider: Arc<Provider<P>>, token_a: Address, token_b: Address, tick_spacing: i32) -> Result<Self> {
        let token_a_decimals = Self::get_decimals(provider.clone(), token_a).await?;
        let token_b_decimals = Self::get_decimals(provider.clone(), token_b).await?;
        let contract = BullaPool::new(address, provider.clone());
        println!("Provider set for pool at {:?}", address);
        Ok(Self {
            address,
            token_a,
            token_b,
            token_a_decimals,
            token_b_decimals,
            tick_spacing,
            contract: Some(contract),
        })
    }

    pub async fn get_adjusted_current_price_and_tick(&self) -> Result<(f64, i32)> {
        if let Some(contract) = &self.contract {
            let state = contract.get_state_of_amm().await?;
            let sqrt_price = state.sqrt_price;
            // Convert sqrt price (Q96.64) to actual price
            let price = (sqrt_price.as_u128() as f64 / (1u128 << 96) as f64).powi(2);
            // Adjust for token decimals (ex: 18 - 6 = 12 decimals difference)
            let adjusted_price = price * (10.0_f64.powi(self.token_a_decimals as i32 - self.token_b_decimals as i32));
            Ok((adjusted_price, state.tick))
        } else {
            anyhow::bail!("Contract not set for pool")
        }
    }

    pub async fn get_tick_range(&self, current_tick: i32, num_ticks: u32) -> (i32, i32) {
        let half_num_ticks = (num_ticks / 2) as i32;
        let lower_tick = current_tick - (half_num_ticks * self.tick_spacing);
        let upper_tick = current_tick + (half_num_ticks * self.tick_spacing);
        (lower_tick, upper_tick)
    }

    pub async fn get_decimals(provider: Arc<Provider<P>>, token: Address) -> Result<u8> {
        let contract = ERC20Token::new(token, provider);
        contract.decimals().await
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn token_a(&self) -> Address {
        self.token_a
    }

    pub fn token_b(&self) -> Address {
        self.token_b
    }

    pub fn token_a_decimals(&self) -> u8 {
        self.token_a_decimals
    }

    pub fn token_b_decimals(&self) -> u8 {
        self.token_b_decimals
    }
} 