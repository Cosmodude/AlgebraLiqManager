use anyhow::Result;
use ethers::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

// Contract interface for Bulla pool
#[derive(Clone, Debug)]
pub struct BullaPool<M> {
    contract: ContractInstance<Arc<Arc<M>>, Arc<M>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateOfAMM {
    pub sqrt_price: U256,  // uint160
    pub tick: i32,         // int24
    pub last_fee: u16,
    pub plugin_config: u8,
    pub active_liquidity: U256,  // uint128
    pub next_tick: i32,          // int24
    pub previous_tick: i32,      // int24
}

impl<M: Middleware + 'static> BullaPool<M> {
    pub fn new(address: Address, client: M) -> Self {
        let abi = include_str!("./onchain/abi/BullaPool.json");
        let client = Arc::new(Arc::new(client));
        let contract = Contract::new(
            address,
            serde_json::from_str::<ethers::abi::Abi>(abi).unwrap(),
            client.clone(),
        );
        Self { contract }
    }

    pub async fn get_state_of_amm(&self) -> Result<StateOfAMM> {
        let state: (U256, i32, u16, u8, U256, i32, i32) = self.contract
            .method("safelyGetStateOfAMM", ())?
            .call()
            .await?;
        
        Ok(StateOfAMM {
            sqrt_price: state.0,
            tick: state.1,
            last_fee: state.2,
            plugin_config: state.3,
            active_liquidity: state.4,
            next_tick: state.5,
            previous_tick: state.6,
        })
    }

    pub async fn get_current_price(&self) -> Result<U256> {
        let state = self.get_state_of_amm().await?;
        Ok(state.sqrt_price)
    }
}

#[derive(Debug, Clone)]
pub struct Pool<P> {
    address: Address,
    token_a: Address,
    token_b: Address,
    tick_spacing: i32,
    contract: Option<BullaPool<Arc<Provider<P>>>>,
}

impl<P: JsonRpcClient + 'static> Pool<P> {
    pub fn new(address: Address, token_a: Address, token_b: Address, tick_spacing: i32) -> Self {
        Self {
            address,
            token_a,
            token_b,
            tick_spacing,
            contract: None,
        }
    }

    pub fn set_provider(&mut self, provider: Provider<P>) {
        let contract = BullaPool::new(self.address, Arc::new(provider));
        self.contract = Some(contract);
        println!("Provider set for pool at {:?}", self.address);
    }

    pub async fn get_current_price(&self, provider: &Provider<P>) -> Result<(f64, i32)> {
        if let Some(contract) = &self.contract {
            let state = contract.get_state_of_amm().await?;
            let sqrt_price = state.sqrt_price;
            // Convert sqrt price (Q96.64) to actual price
            let price = (sqrt_price.as_u128() as f64 / (1u128 << 96) as f64).powi(2);
            // Adjust for token decimals (18 - 6 = 12 decimals difference)
            let adjusted_price = price * 1e12;
            Ok((adjusted_price, state.tick))
        } else {
            anyhow::bail!("Provider not set for pool")
        }
    }

    pub async fn get_tick_range(&self, current_tick: i32, num_ticks: u32) -> (i32, i32) {
        let half_ticks = (num_ticks / 2) as i32;
        let lower_tick = current_tick - (half_ticks * self.tick_spacing);
        let upper_tick = current_tick + (half_ticks * self.tick_spacing);
        (lower_tick, upper_tick)
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
} 