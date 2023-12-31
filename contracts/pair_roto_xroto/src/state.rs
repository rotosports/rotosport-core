use rotosports_pair_bonded::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api};

/// This structure stores a ROTO-xROTO pool's params.
#[cw_serde]
pub struct Params {
    /// ROTO token contract address.
    pub roto_addr: Addr,
    /// xROTO token contract address.
    pub xroto_addr: Addr,
    /// Rotosports Staking contract address.
    pub staking_addr: Addr,
}

/// This structure stores a ROTO-xROTO pool's init params.
#[cw_serde]
pub struct InitParams {
    /// ROTO token contract address.
    pub roto_addr: String,
    /// xROTO token contract address.
    pub xroto_addr: String,
    /// Rotosports Staking contract address.
    pub staking_addr: String,
}

impl InitParams {
    pub fn try_into_params(self, api: &dyn Api) -> Result<Params, ContractError> {
        Ok(Params {
            roto_addr: api.addr_validate(&self.roto_addr)?,
            xroto_addr: api.addr_validate(&self.xroto_addr)?,
            staking_addr: api.addr_validate(&self.staking_addr)?,
        })
    }
}

/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[cw_serde]
pub struct MigrateMsg {}
