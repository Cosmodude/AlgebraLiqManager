use anyhow::Result;
use ethers::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::pool::Pool;
use log::error;

pub struct PriceTracker<P> {
    pool: Arc<Pool<P>>,
    provider: Provider<P>,
    current_price: RwLock<f64>,
    current_tick: RwLock<i32>,
    last_update: RwLock<u64>,
}

impl<P: JsonRpcClient + 'static> PriceTracker<P> {
    pub fn new(pool: Arc<Pool<P>>, provider: Provider<P>) -> Self {
        Self {
            pool,
            provider,
            current_price: RwLock::new(0.0),
            current_tick: RwLock::new(0),
            last_update: RwLock::new(0),
        }
    }

    pub async fn start_tracking(&self) -> Result<()> {
        println!("Starting price tracking for pool {:?}", self.pool.address());
        
        loop {
            match self.update_price().await {
                Ok((price, tick, timestamp)) => {
                    println!("Price: {:.6}, Tick: {}, Timestamp: {}", price, tick, timestamp);
                }
                Err(e) => {
                    error!("Failed to update price: {}", e);
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    }

    async fn update_price(&self) -> Result<(f64, i32, u64)> {
        let (price, tick) = self.pool.get_adjusted_current_price_and_tick(&self.provider).await?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        {
            let mut price_lock = self.current_price.write().await;
            *price_lock = price;
        }
        {
            let mut tick_lock = self.current_tick.write().await;
            *tick_lock = tick;
        }
        {
            let mut time_lock = self.last_update.write().await;
            *time_lock = timestamp;
        }

        Ok((price, tick, timestamp))
    }

    pub async fn get_current_price(&self) -> f64 {
        *self.current_price.read().await
    }

    pub async fn get_current_tick(&self) -> i32 {
        *self.current_tick.read().await
    }

    pub async fn get_last_update(&self) -> u64 {
        *self.last_update.read().await
    }
} 