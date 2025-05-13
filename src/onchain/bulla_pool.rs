use serde::Deserialize; 
use ethers::prelude::*;
use std::sync::Arc;
use anyhow::Result;

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
        let abi = include_str!("./abi/BullaPool.json");
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