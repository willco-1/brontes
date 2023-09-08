use reth_primitives::{Address, Log, U256};
use reth_rpc_types::trace::parity::TransactionTrace;

#[derive(Debug, Clone)]
pub enum Actions {
    Swap(NormalizedSwap),
    Transfer(NormalizedTransfer),
    Mint(NormalizedMint),
    Burn(NormalizedBurn),
    Unclassified(TransactionTrace, Vec<Log>),
}

impl Actions {
    pub fn get_logs(&self) -> Vec<Log> {
        match self {
            Self::Unclassified(_, log) => log.clone(),
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct NormalizedSwap {
    pub address: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub amount_out: U256,
}

#[derive(Debug, Clone)]
pub struct NormalizedTransfer {
    pub from: Address,
    pub to: Address,
    pub token: Address,
    pub amount: U256,
}

#[derive(Debug, Clone)]
pub struct NormalizedMint {
    pub from: Vec<Address>,
    pub to: Vec<Address>,
    pub token: Vec<Address>,
    pub amount: Vec<U256>,
}

#[derive(Debug, Clone)]
pub struct NormalizedBurn {
    pub from: Vec<Address>,
    pub to: Vec<Address>,
    pub token: Vec<Address>,
    pub amount: Vec<U256>,
}

pub trait NormalizedAction: Send + Clone {
    fn get_action(&self) -> &Actions;
}

impl NormalizedAction for Actions {
    fn get_action(&self) -> &Actions {
        &self
    }
}