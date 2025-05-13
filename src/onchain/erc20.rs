use ethers::prelude::*;
use ethers::abi::Abi;
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ERC20Token<M> {
    contract: Contract<Arc<M>>,
}

impl<M: Middleware + 'static> ERC20Token<M> {
    pub fn new(address: Address, client: Arc<M>) -> Self {
        let abi = include_str!("./abi/ERC20.json");
        let parsed_abi: Abi = serde_json::from_str(abi).expect("Invalid ERC20 ABI");
        let contract = Contract::new(address, parsed_abi, Arc::new(client));
        Self { contract }
    }

    pub async fn decimals(&self) -> Result<u8> {
        let decimals: u8 = self.contract.method("decimals", ())?.call().await?;
        Ok(decimals)
    }

    pub async fn approve(&self, spender: Address, amount: U256) -> Result<H256> {
        let approve_call = self.contract
            .method::<_, H256>("approve", (spender, amount))?;
        let tx = approve_call.send().await?;
        Ok(tx.tx_hash())
    }
} 