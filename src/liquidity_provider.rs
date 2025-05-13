use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;
use crate::pool::Pool;
use crate::onchain::erc20::ERC20Token;
use crate::onchain::liquidity_manager::LiquidityManager;
use log::info;

pub struct LiquidityProvider<P> {
    pool: Arc<Pool<P>>,
    provider: Provider<P>,
    num_ticks: u32,
    liquidity_manager: LiquidityManager<P>,
}

impl<P: JsonRpcClient + Clone + 'static> LiquidityProvider<P> {
    pub fn new(pool: Arc<Pool<P>>, provider: Provider<P>, num_ticks: u32, liquidity_manager_contract: Address) -> Result<Self> {
        let liquidity_manager = LiquidityManager::new(provider.clone(), liquidity_manager_contract)?;
        
        Ok(Self {
            pool,
            provider,
            num_ticks,
            liquidity_manager,
        })
    }

    pub async fn provide_liquidity(&self, amount_a: U256, amount_b: U256) -> Result<()> {
        let (current_price, current_tick) = self.pool.get_adjusted_current_price_and_tick(&self.provider).await?;
        let (bottom_tick, top_tick) = self.pool.get_tick_range(current_tick, self.num_ticks).await;
        
        info!("Providing liquidity at price {:.6}, tick range [{}, {}]", 
            current_price, bottom_tick, top_tick);

        let token_a = self.pool.token_a();
        let token_b = self.pool.token_b();

        let provider = Arc::new(self.provider.clone());
        let token_a_contract = ERC20Token::new(token_a, provider.clone());
        let token_b_contract = ERC20Token::new(token_b, provider.clone());

        // Approve tokens
        token_a_contract.approve(self.liquidity_manager.address, amount_a).await?;
        token_b_contract.approve(self.liquidity_manager.address, amount_b).await?;

        self.liquidity_manager.provide_liquidity(
            bottom_tick,
            top_tick,
            amount_a,
            amount_b,
            current_tick,
        ).await?;

        Ok(())
    }

    pub async fn remove_liquidity(&self, position_id: U256) -> Result<()> {
        let bottom_tick = -100;
        let top_tick = 100;
        let amount = U128::from(position_id.as_u128());
        let amount0_min = U256::from(0);
        let amount1_min = U256::from(0);

        self.liquidity_manager.withdraw_liquidity(
            bottom_tick,
            top_tick,
            amount,
            amount0_min,
            amount1_min,
        ).await?;

        Ok(())
    }

    fn tick_to_sqrt_price(tick: i32) -> f64 {
        1.0001_f64.powf(tick as f64 / 2.0)
    }
} 