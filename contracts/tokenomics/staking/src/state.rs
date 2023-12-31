use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// This structure stores the main parameters for the staking contract.
#[cw_serde]
pub struct Config {
    /// The ROTO token contract address
    pub roto_token_addr: Addr,
    /// The xROTO token contract address
    pub xroto_token_addr: Addr,
}

/// Stores the contract config at the given key
pub const CONFIG: Item<Config> = Item::new("config");
