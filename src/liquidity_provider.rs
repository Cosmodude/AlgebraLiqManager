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
        // 1. Calculate optimal tick range based on current price
        let (current_price, current_tick) = self.pool.get_current_price(&self.provider).await?;
        let (bottom_tick, top_tick) = self.pool.get_tick_range(current_tick, self.num_ticks).await;
        
        info!("Providing liquidity at price {:.6}, tick range [{}, {}]", 
            current_price, bottom_tick, top_tick);

        // 2. Approve tokens for the liquidity manager
        let token_a = self.pool.token_a();
        let token_b = self.pool.token_b();

        // Create ERC20 contract instances
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

        // 3. Call liquidity manager to provide liquidity
        let liq_manager_abi: ethers::abi::Abi = serde_json::from_str(include_str!("../contracts/out/LiqManager.sol/LiquidityManager.json"))?;
        let liq_manager_contract = Contract::new(
            self.liquidity_manager_contract,
            liq_manager_abi,
            provider
        );

        // Calculate liquidity amount based on amounts and price
        let liquidity_desired = U128::from(amount_a.as_u128() + amount_b.as_u128());
        let data = Vec::<u8>::new();

        // Call provideLiquidity function
        let provide_call = liq_manager_contract
            .method::<_, H256>(
                "provideLiquidity",
                (
                    self.liquidity_manager_contract, // recipient
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
        // 1. Call liquidity manager to remove liquidity
        let liq_manager_abi: ethers::abi::Abi = serde_json::from_str(include_str!("../contracts/out/LiqManager.sol/LiquidityManager.json"))?;
        let provider = Arc::new(self.provider.clone());
        let liq_manager_contract = Contract::new(
            self.liquidity_manager_contract,
            liq_manager_abi,
            provider
        );

        // Get position details from the position ID
        // Note: This is a simplified version. In a real implementation,
        // you would need to decode the position ID to get bottom_tick and top_tick
        let bottom_tick = -100; // Example value
        let top_tick = 100;     // Example value
        let amount = U128::from(position_id.as_u128());
        let amount0_min = U256::from(0); // No minimum amount required
        let amount1_min = U256::from(0); // No minimum amount required

        // Call withdrawLiquidity function
        let withdraw_call = liq_manager_contract
            .method::<_, H256>(
                "withdrawLiquidity",
                (
                    self.liquidity_manager_contract, // recipient
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
} 