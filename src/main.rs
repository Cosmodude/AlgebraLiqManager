mod pool;
mod price_tracker;
mod liquidity_provider;
mod rebalancer;

use anyhow::Result;
use log::{info, error};
use std::sync::Arc;
use ethers::prelude::*;
use crate::pool::Pool;
use crate::liquidity_provider::LiquidityProvider;
use price_tracker::PriceTracker;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    dotenv().ok();
    info!("Starting Bulla Liquidity Manager");

    // Load configuration
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let pool_address = env::var("POOL_ADDRESS").expect("POOL_ADDRESS must be set");
    let token_a = env::var("TOKEN_A").expect("TOKEN_A must be set");
    let token_b = env::var("TOKEN_B").expect("TOKEN_B must be set");
    let tick_spacing = env::var("TICK_SPACING")
        .expect("TICK_SPACING must be set")
        .parse::<i32>()?;
    let liquidity_manager = env::var("LIQUIDITY_MANAGER_CONTRACT").expect("LIQUIDITY_MANAGER must be set");

    // Connect to Berachain node
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let pool_address: Address = pool_address.parse()?;
    let token_a: Address = token_a.parse()?;
    let token_b: Address = token_b.parse()?;
    let liquidity_manager: Address = liquidity_manager.parse()?;

    // Create pool instance
    let mut pool = Pool::new(pool_address, token_a, token_b, tick_spacing);
    pool.set_provider(provider.clone());
    let pool = Arc::new(pool);

    // Create liquidity provider
    let liquidity_provider = Arc::new(LiquidityProvider::new(
        pool.clone(),
        provider.clone(),
        10, // num_ticks
        liquidity_manager,
    ));

    // Create price tracker
    let price_tracker = Arc::new(PriceTracker::new(pool.clone(), provider.clone()));

    // Start price tracking
    let price_tracker_clone = price_tracker.clone();
    tokio::spawn(async move {
        if let Err(e) = price_tracker_clone.start_tracking().await {
            error!("Price tracking failed: {}", e);
        }
    });

    // Keep the main thread alive
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
