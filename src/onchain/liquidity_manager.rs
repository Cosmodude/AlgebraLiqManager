use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;
use log::info;

pub struct LiquidityManager<P> {
    contract: Contract<Arc<Arc<Provider<P>>>>,
    pub address: Address,
}

impl<P: JsonRpcClient + Clone + 'static> LiquidityManager<P> {
    pub fn new(provider: Provider<P>, address: Address) -> Result<Self> {
        let abi: ethers::abi::Abi = serde_json::from_str(include_str!("../../contracts/out/LiqManager.sol/LiquidityManager.json"))?;
        let contract = Contract::new(address, abi, Arc::new(Arc::new(Arc::new(provider))));
        
        Ok(Self {
            contract,
            address,
        })
    }

    fn calculate_liquidity(
        amount_a: U256,
        amount_b: U256,
        bottom_tick: i32,
        top_tick: i32,
        current_tick: i32,
    ) -> U128 {
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
        U128::from(liquidity as u128)
    }

    fn tick_to_sqrt_price(tick: i32) -> f64 {
        1.0001_f64.powf(tick as f64 / 2.0)
    }

    pub async fn provide_liquidity(
        &self,
        bottom_tick: i32,
        top_tick: i32,
        amount_a: U256,
        amount_b: U256,
        current_tick: i32,
    ) -> Result<H256> {
        let liquidity_desired = Self::calculate_liquidity(amount_a, amount_b, bottom_tick, top_tick, current_tick);
        let data = Vec::<u8>::new();

        let provide_call = self.contract
            .method::<_, H256>(
                "provideLiquidity",
                (
                    self.address,
                    bottom_tick,
                    top_tick,
                    liquidity_desired,
                    data,
                ),
            )?;
        let tx = provide_call.send().await?;
        info!("Liquidity provided successfully. Transaction hash: {:?}", tx.tx_hash());
        Ok(tx.tx_hash())
    }

    pub async fn withdraw_liquidity(
        &self,
        bottom_tick: i32,
        top_tick: i32,
        amount: U128,
        amount0_min: U256,
        amount1_min: U256,
    ) -> Result<H256> {
        let withdraw_call = self.contract
            .method::<_, H256>(
                "withdrawLiquidity",
                (
                    self.address,
                    bottom_tick,
                    top_tick,
                    amount,
                    amount0_min,
                    amount1_min,
                ),
            )?;
        let tx = withdraw_call.send().await?;
        info!("Liquidity removed successfully. Transaction hash: {:?}", tx.tx_hash());
        Ok(tx.tx_hash())
    }
} 