use rotosports::asset::{Asset, AssetInfo, PairInfo};
use rotosports::factory::{InstantiateMsg as FactoryInstantiateMsg, PairConfig, PairType};
use rotosports::pair::{
    ConfigResponse, Cw20HookMsg, InstantiateMsg as PairInstantiateMsg, ReverseSimulationResponse,
    SimulationResponse,
};
use rotosports::staking::{
    ConfigResponse as StakingConfigResponse, InstantiateMsg as StakingInstantiateMsg,
    QueryMsg as StakingQueryMsg,
};

use rotosports::pair_bonded::{ExecuteMsg, QueryMsg};
use rotosports::token::InstantiateMsg as TokenInstantiateMsg;
use rotosports_pair_roto_xroto::state::Params;
use cosmwasm_std::{to_binary, Addr, Coin, Uint128};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw_multi_test::{App, ContractWrapper, Executor};

struct RotosportsContracts {
    factory_instance: Addr,
    pair_instance: Addr,
    roto_instance: Addr,
    xroto_instance: Addr,
}

fn mock_app(owner: Addr, coins: Vec<Coin>) -> App {
    App::new(|router, _, storage| router.bank.init_balance(storage, &owner, coins).unwrap())
}

fn store_pair_code(app: &mut App) -> u64 {
    let pair_contract = Box::new(ContractWrapper::new_with_empty(
        rotosports_pair_roto_xroto::execute,
        rotosports_pair_roto_xroto::instantiate,
        rotosports_pair_roto_xroto::query,
    ));

    app.store_code(pair_contract)
}

fn store_staking_code(app: &mut App) -> u64 {
    let staking_contract = Box::new(
        ContractWrapper::new_with_empty(
            rotosports_staking::contract::execute,
            rotosports_staking::contract::instantiate,
            rotosports_staking::contract::query,
        )
        .with_reply_empty(rotosports_staking::contract::reply),
    );

    app.store_code(staking_contract)
}

fn store_roto_code(app: &mut App) -> u64 {
    let roto_contract = Box::new(ContractWrapper::new_with_empty(
        rotosports_token::contract::execute,
        rotosports_token::contract::instantiate,
        rotosports_token::contract::query,
    ));

    app.store_code(roto_contract)
}

fn store_xroto_code(app: &mut App) -> u64 {
    let xroto_contract = Box::new(ContractWrapper::new_with_empty(
        rotosports_xroto_token::contract::execute,
        rotosports_xroto_token::contract::instantiate,
        rotosports_xroto_token::contract::query,
    ));

    app.store_code(xroto_contract)
}

fn store_factory_code(app: &mut App) -> u64 {
    let factory_contract = Box::new(ContractWrapper::new_with_empty(
        rotosports_factory::contract::execute,
        rotosports_factory::contract::instantiate,
        rotosports_factory::contract::query,
    ));

    app.store_code(factory_contract)
}

fn instantiate_factory_contract(app: &mut App, owner: Addr, pair_code_id: u64) -> Addr {
    let code = store_factory_code(app);

    let msg = FactoryInstantiateMsg {
        pair_configs: vec![PairConfig {
            code_id: pair_code_id,
            maker_fee_bps: 0,
            total_fee_bps: 0,
            pair_type: PairType::Custom("bonded".to_string()),
            is_disabled: false,
            is_generator_disabled: false,
        }],
        token_code_id: 0,
        fee_address: None,
        generator_address: None,
        owner: owner.to_string(),
        whitelist_code_id: 234u64,
        coin_registry_address: "coin_registry".to_owned(),
    };

    app.instantiate_contract(
        code,
        owner,
        &msg,
        &[],
        String::from("Rotosports Factory"),
        None,
    )
    .unwrap()
}

fn instantiate_token(app: &mut App, owner: Addr) -> Addr {
    let token_code_id = store_roto_code(app);

    let msg = TokenInstantiateMsg {
        name: "Rotosports Token".to_string(),
        symbol: "ROTO".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    app.instantiate_contract(
        token_code_id,
        owner.clone(),
        &msg,
        &[],
        String::from("Rotosports Token"),
        None,
    )
    .unwrap()
}

fn instantiate_staking(app: &mut App, owner: Addr, token_instance: &Addr) -> (Addr, Addr) {
    let xroto_code_id = store_xroto_code(app);
    let staking_code_id = store_staking_code(app);

    let msg = StakingInstantiateMsg {
        owner: owner.to_string(),
        token_code_id: xroto_code_id,
        deposit_token_addr: token_instance.to_string(),
        marketing: None,
    };

    let staking_instance = app
        .instantiate_contract(
            staking_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("Rotosports Staking"),
            None,
        )
        .unwrap();

    let resp: StakingConfigResponse = app
        .wrap()
        .query_wasm_smart(&staking_instance, &StakingQueryMsg::Config {})
        .unwrap();

    (staking_instance, resp.share_token_addr)
}

fn instantiate_rotosports(mut router: &mut App, owner: &Addr) -> RotosportsContracts {
    let pair_code_id = store_pair_code(&mut router);

    let factory_instance = instantiate_factory_contract(router, owner.clone(), pair_code_id);
    let token_instance = instantiate_token(router, owner.clone());

    let (staking_instance, xroto_instance) =
        instantiate_staking(router, owner.clone(), &token_instance);

    let msg = PairInstantiateMsg {
        asset_infos: vec![
            AssetInfo::Token {
                contract_addr: token_instance.clone(),
            },
            AssetInfo::Token {
                contract_addr: xroto_instance.clone(),
            },
        ],
        token_code_id: 123,
        factory_addr: factory_instance.to_string(),
        init_params: Some(
            to_binary(&Params {
                roto_addr: token_instance.clone(),
                xroto_addr: xroto_instance.clone(),
                staking_addr: staking_instance.clone(),
            })
            .unwrap(),
        ),
    };

    let pair_instance = router
        .instantiate_contract(
            pair_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("ROTO-xROTO pair"),
            None,
        )
        .unwrap();

    RotosportsContracts {
        pair_instance,
        roto_instance: token_instance,
        xroto_instance,
        factory_instance,
    }
}

fn mint_tokens(router: &mut App, owner: Addr, token_addr: Addr, amount: Uint128, to: Addr) {
    router
        .execute_contract(
            owner,
            token_addr,
            &Cw20ExecuteMsg::Mint {
                recipient: to.to_string(),
                amount,
            },
            &[],
        )
        .unwrap();
}

fn assert_user_balance(router: &mut App, token: &Addr, user: &Addr, expected_balance: u64) {
    let balance: cw20::BalanceResponse = router
        .wrap()
        .query_wasm_smart(
            token,
            &Cw20QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap();
    assert_eq!(balance.balance, Uint128::from(expected_balance));
}

#[test]
fn test_pair_instantiation() {
    let owner = Addr::unchecked("owner");

    let mut router = mock_app(owner.clone(), vec![]);

    let pair_code_id = store_pair_code(&mut router);

    let factory_instance = instantiate_factory_contract(&mut router, owner.clone(), pair_code_id);
    let token_instance = instantiate_token(&mut router, owner.clone());

    let (staking_instance, xroto_instance) =
        instantiate_staking(&mut router, owner.clone(), &token_instance);

    let msg = PairInstantiateMsg {
        asset_infos: vec![
            AssetInfo::Token {
                contract_addr: token_instance.clone(),
            },
            AssetInfo::Token {
                contract_addr: xroto_instance.clone(),
            },
        ],
        token_code_id: 123,
        factory_addr: factory_instance.to_string(),
        init_params: None,
    };

    let err = router
        .instantiate_contract(
            pair_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("ROTO-xROTO pair"),
            None,
        )
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        "You need to provide init params".to_string()
    );

    let msg = PairInstantiateMsg {
        asset_infos: vec![
            AssetInfo::Token {
                contract_addr: token_instance.clone(),
            },
            AssetInfo::Token {
                contract_addr: xroto_instance.clone(),
            },
        ],
        token_code_id: 123,
        factory_addr: factory_instance.to_string(),
        init_params: Some(
            to_binary(&Params {
                roto_addr: token_instance.clone(),
                xroto_addr: xroto_instance.clone(),
                staking_addr: staking_instance.clone(),
            })
            .unwrap(),
        ),
    };

    let pair_instance = router
        .instantiate_contract(
            pair_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("ROTO-xROTO pair"),
            None,
        )
        .unwrap();

    assert_eq!(factory_instance.to_string(), "contract0");
    assert_eq!(token_instance.to_string(), "contract1");
    assert_eq!(staking_instance.to_string(), "contract2");
    assert_eq!(xroto_instance.to_string(), "contract3");
    assert_eq!(pair_instance.to_string(), "contract4");
}

#[test]
fn test_pair_swap() {
    let owner = Addr::unchecked("owner");

    let user1 = Addr::unchecked("user1");
    let user2 = Addr::unchecked("user2");

    let mut router = mock_app(owner.clone(), vec![]);

    let contracts = instantiate_rotosports(&mut router, &owner);

    // Mint ROTO
    mint_tokens(
        &mut router,
        owner.clone(),
        contracts.roto_instance.clone(),
        Uint128::from(10_000u64),
        user1.clone(),
    );
    mint_tokens(
        &mut router,
        owner.clone(),
        contracts.roto_instance.clone(),
        Uint128::from(30_000u64),
        user2.clone(),
    );

    // Test simulate and reverse simulate with empty staking (ROTO->xROTO)
    let res: SimulationResponse = router
        .wrap()
        .query_wasm_smart(
            &contracts.pair_instance,
            &QueryMsg::Simulation {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: contracts.roto_instance.clone(),
                    },
                    amount: Uint128::from(10_000u64),
                },
            },
        )
        .unwrap();
    assert_eq!(
        res,
        SimulationResponse {
            return_amount: Uint128::from(10000u64),
            spread_amount: Uint128::zero(),
            commission_amount: Uint128::zero()
        }
    );
    let res: ReverseSimulationResponse = router
        .wrap()
        .query_wasm_smart(
            &contracts.pair_instance,
            &QueryMsg::ReverseSimulation {
                ask_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: contracts.xroto_instance.clone(),
                    },
                    amount: Uint128::from(10_000u64),
                },
            },
        )
        .unwrap();
    assert_eq!(
        res,
        ReverseSimulationResponse {
            offer_amount: Uint128::from(10000u64),
            spread_amount: Uint128::zero(),
            commission_amount: Uint128::zero()
        }
    );

    // Test Swap operation ROTO->xROTO
    router
        .execute_contract(
            user1.clone(),
            contracts.roto_instance.clone(),
            &Cw20ExecuteMsg::Send {
                contract: contracts.pair_instance.clone().to_string(),
                amount: Uint128::from(10_000u64),
                msg: to_binary(&Cw20HookMsg::Swap {
                    ask_asset_info: None,
                    belief_price: None,
                    max_spread: None,
                    to: None,
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap();
    assert_user_balance(&mut router, &contracts.xroto_instance, &user1, 9_000u64);

    router
        .execute_contract(
            user2.clone(),
            contracts.roto_instance.clone(),
            &Cw20ExecuteMsg::Send {
                contract: contracts.pair_instance.clone().to_string(),
                amount: Uint128::from(30_000u64),
                msg: to_binary(&Cw20HookMsg::Swap {
                    ask_asset_info: None,
                    belief_price: None,
                    max_spread: None,
                    to: None,
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap();
    assert_user_balance(&mut router, &contracts.xroto_instance, &user2, 30_000u64);

    // Test simulate and reverse simulate (ROTO->xROTO)
    let res: SimulationResponse = router
        .wrap()
        .query_wasm_smart(
            &contracts.pair_instance,
            &QueryMsg::Simulation {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: contracts.roto_instance.clone(),
                    },
                    amount: Uint128::from(10_000u64),
                },
            },
        )
        .unwrap();
    assert_eq!(
        res,
        SimulationResponse {
            return_amount: Uint128::from(10000u64),
            spread_amount: Uint128::zero(),
            commission_amount: Uint128::zero()
        }
    );
    let res: ReverseSimulationResponse = router
        .wrap()
        .query_wasm_smart(
            &contracts.pair_instance,
            &QueryMsg::ReverseSimulation {
                ask_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: contracts.xroto_instance.clone(),
                    },
                    amount: Uint128::from(10_000u64),
                },
            },
        )
        .unwrap();
    assert_eq!(
        res,
        ReverseSimulationResponse {
            offer_amount: Uint128::from(10000u64),
            spread_amount: Uint128::zero(),
            commission_amount: Uint128::zero()
        }
    );

    // Test simulate and reverse simulate (xROTO->ROTO)
    let res: SimulationResponse = router
        .wrap()
        .query_wasm_smart(
            &contracts.pair_instance,
            &QueryMsg::Simulation {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: contracts.xroto_instance.clone(),
                    },
                    amount: Uint128::from(10_000u64),
                },
            },
        )
        .unwrap();
    assert_eq!(
        res,
        SimulationResponse {
            return_amount: Uint128::from(10000u64),
            spread_amount: Uint128::zero(),
            commission_amount: Uint128::zero()
        }
    );
    let res: ReverseSimulationResponse = router
        .wrap()
        .query_wasm_smart(
            &contracts.pair_instance,
            &QueryMsg::ReverseSimulation {
                ask_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: contracts.roto_instance.clone(),
                    },
                    amount: Uint128::from(10_000u64),
                },
            },
        )
        .unwrap();
    assert_eq!(
        res,
        ReverseSimulationResponse {
            offer_amount: Uint128::from(10000u64),
            spread_amount: Uint128::zero(),
            commission_amount: Uint128::zero()
        }
    );

    // Test Swap operation ROTO->xROTO
    router
        .execute_contract(
            user1.clone(),
            contracts.xroto_instance.clone(),
            &Cw20ExecuteMsg::Send {
                contract: contracts.pair_instance.clone().to_string(),
                amount: Uint128::from(9_000u64),
                msg: to_binary(&Cw20HookMsg::Swap {
                    ask_asset_info: None,
                    belief_price: None,
                    max_spread: None,
                    to: None,
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap();
    assert_user_balance(&mut router, &contracts.roto_instance, &user1, 9_000u64);

    router
        .execute_contract(
            user2.clone(),
            contracts.xroto_instance.clone(),
            &Cw20ExecuteMsg::Send {
                contract: contracts.pair_instance.clone().to_string(),
                amount: Uint128::from(30_000u64),
                msg: to_binary(&Cw20HookMsg::Swap {
                    ask_asset_info: None,
                    belief_price: None,
                    max_spread: None,
                    to: None,
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap();
    assert_user_balance(&mut router, &contracts.roto_instance, &user2, 30_000u64);
}

#[test]
fn test_unsupported_methods() {
    let owner = Addr::unchecked("owner");

    let mut router = mock_app(owner.clone(), vec![]);

    let contracts = instantiate_rotosports(&mut router, &owner);

    // Test provide liquidity
    let err = router
        .execute_contract(
            owner.clone(),
            contracts.pair_instance.clone(),
            &ExecuteMsg::ProvideLiquidity {
                assets: [
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: contracts.roto_instance.clone(),
                        },
                        amount: Uint128::from(100u64),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: contracts.xroto_instance.clone(),
                        },
                        amount: Uint128::from(100u64),
                    },
                ],
                slippage_tolerance: None,
                auto_stake: None,
                receiver: None,
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Operation is not supported for this pool."
    );

    // Test update config
    let err = router
        .execute_contract(
            owner.clone(),
            contracts.pair_instance.clone(),
            &ExecuteMsg::UpdateConfig {
                params: Default::default(),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Operation is not supported for this pool."
    );

    // Test update config
    let err = router
        .execute_contract(
            owner.clone(),
            contracts.pair_instance.clone(),
            &ExecuteMsg::UpdateConfig {
                params: Default::default(),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Operation is not supported for this pool."
    );

    // Test native-swap
    let err = router
        .execute_contract(
            owner.clone(),
            contracts.pair_instance.clone(),
            &ExecuteMsg::Swap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: contracts.roto_instance.clone(),
                    },
                    amount: Uint128::from(10u8),
                },
                belief_price: None,
                max_spread: None,
                to: None,
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Operation is not supported for this pool."
    );
}

#[test]
fn test_queries() {
    let owner = Addr::unchecked("owner");

    let mut router = mock_app(owner.clone(), vec![]);

    let contracts = instantiate_rotosports(&mut router, &owner);

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(&contracts.pair_instance, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(
        res,
        ConfigResponse {
            block_time_last: 0u64,
            params: None,
            owner,
            factory_addr: contracts.factory_instance
        }
    );

    let res: PairInfo = router
        .wrap()
        .query_wasm_smart(&contracts.pair_instance, &QueryMsg::Pair {})
        .unwrap();
    assert_eq!(
        res,
        PairInfo {
            asset_infos: vec![
                AssetInfo::Token {
                    contract_addr: contracts.roto_instance.clone()
                },
                AssetInfo::Token {
                    contract_addr: contracts.xroto_instance.clone()
                }
            ],
            contract_addr: contracts.pair_instance.clone(),
            liquidity_token: Addr::unchecked(""),
            pair_type: PairType::Custom("Bonded".to_string())
        }
    );
}
