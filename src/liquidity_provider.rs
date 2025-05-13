use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;
use crate::pool::Pool;
use log::{info, error};

pub struct LiquidityProvider<P> {
    pool: Arc<Pool<P>>,
    provider: Provider<P>,
    num_ticks: u32,
    liquidity_manager_contract: Address,
}

impl<P: JsonRpcClient + Clone + 'static> LiquidityProvider<P> {
    pub fn new(pool: Arc<Pool<P>>, provider: Provider<P>, num_ticks: u32, liquidity_manager_contract: Address) -> Self {
        Self {
            pool,
            provider,
            num_ticks,
            liquidity_manager_contract,
        }
    }

    pub async fn provide_liquidity(&self, amount_a: U256, amount_b: U256) -> Result<()> {
        let (current_price, current_tick) = self.pool.get_current_price(&self.provider).await?;
        let (bottom_tick, top_tick) = self.pool.get_tick_range(current_tick, self.num_ticks).await;
        
        info!("Providing liquidity at price {:.6}, tick range [{}, {}]", 
            current_price, bottom_tick, top_tick);

        let token_a = self.pool.token_a();
        let token_b = self.pool.token_b();

        let erc20_abi: ethers::abi::Abi = serde_json::from_str(include_str!("./onchain/abi/ERC20.json"))?;
        let provider = Arc::new(self.provider.clone());
        let token_a_contract = Contract::new(
            token_a,
            erc20_abi.clone(),
            provider.clone()
        );
        let token_b_contract = Contract::new(
            token_b,
            erc20_abi,
            provider.clone()
        );

        token_a_contract
            .method::<_, H256>("approve", (self.liquidity_manager_contract, amount_a))?
            .send()
            .await?;
        token_b_contract
            .method::<_, H256>("approve", (self.liquidity_manager_contract, amount_b))?
            .send()
            .await?;

        let liq_manager_abi: ethers::abi::Abi = serde_json::from_str(include_str!("../contracts/out/LiqManager.sol/LiquidityManager.json"))?;
        let liq_manager_contract = Contract::new(
            self.liquidity_manager_contract,
            liq_manager_abi,
            provider
        );

        let sqrt_lower = Self::tick_to_sqrt_price(bottom_tick);
        let sqrt_upper = Self::tick_to_sqrt_price(top_tick);
        let sqrt_current = Self::tick_to_sqrt_price(current_tick);
      
        let liquidity_from_token0 = (amount_a.as_u128() as f64) * (sqrt_lower * sqrt_upper) / (sqrt_upper - sqrt_lower);
        let liquidity_from_token1 = (amount_b.as_u128() as f64) / (sqrt_upper - sqrt_lower);
        let liquidity = if sqrt_current <= sqrt_lower {
            liquidity_from_token0
        } else if sqrt_current >= sqrt_upper {
            liquidity_from_token1
        } else {
            liquidity_from_token0.min(liquidity_from_token1)
        };
        let liquidity_desired = U128::from(liquidity as u128);
        let data = Vec::<u8>::new();

        let provide_call = liq_manager_contract
            .method::<_, H256>(
                "provideLiquidity",
                (
                    self.liquidity_manager_contract,
                    bottom_tick,
                    top_tick,
                    liquidity_desired,
                    data,
                ),
            )?;
        let tx = provide_call.send().await?;

        info!("Liquidity provided successfully. Transaction hash: {:?}", tx.tx_hash());
        Ok(())
    }

    pub async fn remove_liquidity(&self, position_id: U256) -> Result<()> {
        let liq_manager_abi: ethers::abi::Abi = serde_json::from_str(include_str!("../contracts/out/LiqManager.sol/LiquidityManager.json"))?;
        let provider = Arc::new(self.provider.clone());
        let liq_manager_contract = Contract::new(
            self.liquidity_manager_contract,
            liq_manager_abi,
            provider
        );

        let bottom_tick = -100;
        let top_tick = 100;
        let amount = U128::from(position_id.as_u128());
        let amount0_min = U256::from(0);
        let amount1_min = U256::from(0);

        let withdraw_call = liq_manager_contract
            .method::<_, H256>(
                "withdrawLiquidity",
                (
                    self.liquidity_manager_contract,
                    bottom_tick,
                    top_tick,
                    amount,
                    amount0_min,
                    amount1_min,
                ),
            )?;
        let tx = withdraw_call.send().await?;

        info!("Liquidity removed successfully. Transaction hash: {:?}", tx.tx_hash());
        Ok(())
    }

    fn tick_to_sqrt_price(tick: i32) -> f64 {
        1.0001_f64.powf(tick as f64 / 2.0)
    }
} 