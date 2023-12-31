use rotosports::asset::{AssetInfo, PairInfo};
use rotosports::factory::PairType;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, Storage, Uint128};
use cw_storage_plus::Item;

use crate::state::{Config, CONFIG};
use crate::state::{PoolParams, PoolState};

pub(crate) fn migrate_config(storage: &mut dyn Storage) -> Result<(), StdError> {
    #[cw_serde]
    pub enum OldPairType {
        /// XYK pair type
        Xyk {},
        /// Stable pair type
        Stable {},
        /// Concentrated pair type
        Concentrated {},
        /// Custom pair type
        Custom(String),
    }

    #[cw_serde]
    pub struct OldPairInfo {
        /// Asset information for the assets in the pool
        pub asset_infos: Vec<AssetInfo>,
        /// Pair contract address
        pub contract_addr: Addr,
        /// Pair LP token address
        pub liquidity_token: Addr,
        /// The pool type (xyk, stableswap etc) available in [`PairType`]
        pub pair_type: OldPairType,
    }

    /// This structure stores the main config parameters for a constant product pair contract.
    #[cw_serde]
    pub struct OldConfig {
        /// The pair information stored in a [`PairInfo`] struct
        pub pair_info: OldPairInfo,
        /// The factory contract address
        pub factory_addr: Addr,
        /// The last timestamp when the pair contract updated the asset cumulative prices
        pub block_time_last: u64,
        /// The vector contains cumulative prices for each pair of assets in the pool
        pub cumulative_prices: Vec<(AssetInfo, AssetInfo, Uint128)>,
        /// Pool parameters
        pub pool_params: PoolParams,
        /// Pool state
        pub pool_state: PoolState,
        /// Pool's owner
        pub owner: Option<Addr>,
    }

    /// Stores the config struct at the given key
    pub const OLD_CONFIG: Item<OldConfig> = Item::new("config");

    let old_config = OLD_CONFIG.load(storage)?;
    let pair_info = PairInfo {
        asset_infos: old_config.pair_info.asset_infos,
        contract_addr: old_config.pair_info.contract_addr,
        liquidity_token: old_config.pair_info.liquidity_token,
        pair_type: PairType::Custom("concentrated".to_string()),
    };

    let new_config = Config {
        pair_info,
        factory_addr: old_config.factory_addr,
        block_time_last: old_config.block_time_last,
        cumulative_prices: old_config.cumulative_prices,
        pool_params: old_config.pool_params,
        pool_state: old_config.pool_state,
        owner: old_config.owner,
        track_asset_balances: false,
    };

    CONFIG.save(storage, &new_config)?;

    Ok(())
}

pub(crate) fn migrate_config_from_v140(storage: &mut dyn Storage) -> Result<(), StdError> {
    /// This structure stores the main config parameters for a constant product pair contract.
    #[cw_serde]
    pub struct OldConfig {
        /// The pair information stored in a [`PairInfo`] struct
        pub pair_info: PairInfo,
        /// The factory contract address
        pub factory_addr: Addr,
        /// The last timestamp when the pair contract updated the asset cumulative prices
        pub block_time_last: u64,
        /// The vector contains cumulative prices for each pair of assets in the pool
        pub cumulative_prices: Vec<(AssetInfo, AssetInfo, Uint128)>,
        /// Pool parameters
        pub pool_params: PoolParams,
        /// Pool state
        pub pool_state: PoolState,
        /// Pool's owner
        pub owner: Option<Addr>,
    }

    /// Stores the config struct at the given key
    pub const OLD_CONFIG: Item<OldConfig> = Item::new("config");

    let old_config = OLD_CONFIG.load(storage)?;

    let new_config = Config {
        pair_info: old_config.pair_info,
        factory_addr: old_config.factory_addr,
        block_time_last: old_config.block_time_last,
        cumulative_prices: old_config.cumulative_prices,
        pool_params: old_config.pool_params,
        pool_state: old_config.pool_state,
        owner: old_config.owner,
        track_asset_balances: false,
    };

    CONFIG.save(storage, &new_config)?;

    Ok(())
}
