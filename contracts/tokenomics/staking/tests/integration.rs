use rotosports::staking::{ConfigResponse, Cw20HookMsg, InstantiateMsg as xInstatiateMsg, QueryMsg};
use rotosports::token::InstantiateMsg;
use cosmwasm_std::{attr, to_binary, Addr, QueryRequest, Uint128, WasmQuery};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw_multi_test::{App, ContractWrapper, Executor};

const ALICE: &str = "alice";
const BOB: &str = "bob";
const CAROL: &str = "carol";
const ATTACKER: &str = "attacker";
const VICTIM: &str = "victim";

#[test]
fn check_deflate_liquidity() {
    let mut router = mock_app();

    let owner = Addr::unchecked("owner");

    let (roto_token_instance, staking_instance, _) =
        instantiate_contracts(&mut router, owner.clone());

    mint_some_roto(
        &mut router,
        owner.clone(),
        roto_token_instance.clone(),
        ATTACKER,
    );

    mint_some_roto(
        &mut router,
        owner.clone(),
        roto_token_instance.clone(),
        VICTIM,
    );

    let attacker_address = Addr::unchecked(ATTACKER);
    let victim_address = Addr::unchecked(VICTIM);

    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Enter {}).unwrap(),
        amount: Uint128::from(1000u128),
    };

    let err = router
        .execute_contract(
            attacker_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Initial stake amount must be more than 1000"
    );

    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Enter {}).unwrap(),
        amount: Uint128::from(1001u128),
    };

    router
        .execute_contract(
            attacker_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap();

    let msg = Cw20ExecuteMsg::Transfer {
        recipient: staking_instance.to_string(),
        amount: Uint128::from(5000u128),
    };

    router
        .execute_contract(
            attacker_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap();

    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Enter {}).unwrap(),
        amount: Uint128::from(2u128),
    };

    let err = router
        .execute_contract(
            victim_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap_err();

    assert_eq!(err.root_cause().to_string(), "Insufficient amount of Stake");

    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Enter {}).unwrap(),
        amount: Uint128::from(10u128),
    };

    router
        .execute_contract(
            victim_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap();
}

fn mock_app() -> App {
    App::default()
}

fn instantiate_contracts(router: &mut App, owner: Addr) -> (Addr, Addr, Addr) {
    let roto_token_contract = Box::new(ContractWrapper::new_with_empty(
        rotosports_token::contract::execute,
        rotosports_token::contract::instantiate,
        rotosports_token::contract::query,
    ));

    let roto_token_code_id = router.store_code(roto_token_contract);

    let msg = InstantiateMsg {
        name: String::from("Roto token"),
        symbol: String::from("ROTO"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let roto_token_instance = router
        .instantiate_contract(
            roto_token_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("ROTO"),
            None,
        )
        .unwrap();

    let staking_contract = Box::new(
        ContractWrapper::new_with_empty(
            rotosports_staking::contract::execute,
            rotosports_staking::contract::instantiate,
            rotosports_staking::contract::query,
        )
        .with_reply_empty(rotosports_staking::contract::reply),
    );

    let staking_code_id = router.store_code(staking_contract);

    let msg = xInstatiateMsg {
        owner: owner.to_string(),
        token_code_id: roto_token_code_id,
        deposit_token_addr: roto_token_instance.to_string(),
        marketing: None,
    };
    let staking_instance = router
        .instantiate_contract(
            staking_code_id,
            owner,
            &msg,
            &[],
            String::from("xROTO"),
            None,
        )
        .unwrap();

    let msg = QueryMsg::Config {};
    let res = router
        .wrap()
        .query::<ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: staking_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }))
        .unwrap();

    // In multitest, contract names are named in the order in which contracts are created.
    assert_eq!("contract0", roto_token_instance);
    assert_eq!("contract1", staking_instance);
    assert_eq!("contract2", res.share_token_addr);

    let x_roto_token_instance = res.share_token_addr;

    (
        roto_token_instance,
        staking_instance,
        x_roto_token_instance,
    )
}

fn mint_some_roto(router: &mut App, owner: Addr, roto_token_instance: Addr, to: &str) {
    let msg = cw20::Cw20ExecuteMsg::Mint {
        recipient: String::from(to),
        amount: Uint128::from(10000u128),
    };
    let res = router
        .execute_contract(owner.clone(), roto_token_instance.clone(), &msg, &[])
        .unwrap();
    assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[1].attributes[2], attr("to", String::from(to)));
    assert_eq!(
        res.events[1].attributes[3],
        attr("amount", Uint128::from(10000u128))
    );
}

#[test]
fn cw20receive_enter_and_leave() {
    let mut router = mock_app();

    let owner = Addr::unchecked("owner");

    let (roto_token_instance, staking_instance, x_roto_token_instance) =
        instantiate_contracts(&mut router, owner.clone());

    // Mint 10000 ROTO for Alice
    mint_some_roto(
        &mut router,
        owner.clone(),
        roto_token_instance.clone(),
        ALICE,
    );

    let alice_address = Addr::unchecked(ALICE);

    // Check if Alice's ROTO balance is 100
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(10000u128)
        }
    );

    // We can unstake ROTO only by calling the xROTO token.
    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Leave {}).unwrap(),
        amount: Uint128::from(10u128),
    };

    let resp = router
        .execute_contract(
            alice_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(resp.root_cause().to_string(), "Unauthorized");

    // Tru to stake Alice's 1100 ROTO for 1100 xROTO
    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Enter {}).unwrap(),
        amount: Uint128::from(1100u128),
    };

    router
        .execute_contract(
            alice_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Check if Alice's xROTO balance is 1100
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(100u128)
        }
    );

    // Check if Alice's ROTO balance is 8900
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(8900u128)
        }
    );

    // Check if the staking contract's ROTO balance is 1100
    let msg = Cw20QueryMsg::Balance {
        address: staking_instance.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(1100u128)
        }
    );

    // We can stake tokens only by calling the ROTO token.
    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Enter {}).unwrap(),
        amount: Uint128::from(10u128),
    };

    let resp = router
        .execute_contract(
            alice_address.clone(),
            x_roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(resp.root_cause().to_string(), "Unauthorized");

    // Try to unstake Alice's 10 xROTO for 10 ROTO
    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Leave {}).unwrap(),
        amount: Uint128::from(10u128),
    };

    router
        .execute_contract(
            alice_address.clone(),
            x_roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Check if Alice's xROTO balance is 90
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(90u128)
        }
    );

    // Check if Alice's ROTO balance is 8910
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(8910u128)
        }
    );

    // Check if the staking contract's ROTO balance is 1090
    let msg = Cw20QueryMsg::Balance {
        address: staking_instance.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(1090u128)
        }
    );

    // Check if the staking contract's xROTO balance is 1000
    let msg = Cw20QueryMsg::Balance {
        address: staking_instance.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(1000u128)
        }
    );

    let res: Uint128 = router
        .wrap()
        .query_wasm_smart(staking_instance.clone(), &QueryMsg::TotalDeposit {})
        .unwrap();
    assert_eq!(res.u128(), 1090);
    let res: Uint128 = router
        .wrap()
        .query_wasm_smart(staking_instance, &QueryMsg::TotalShares {})
        .unwrap();
    assert_eq!(res.u128(), 1090);
}

#[test]
fn should_not_allow_withdraw_more_than_what_you_have() {
    let mut router = mock_app();

    let owner = Addr::unchecked("owner");

    let (roto_token_instance, staking_instance, x_roto_token_instance) =
        instantiate_contracts(&mut router, owner.clone());

    // Mint 10000 ROTO for Alice
    mint_some_roto(
        &mut router,
        owner.clone(),
        roto_token_instance.clone(),
        ALICE,
    );
    let alice_address = Addr::unchecked(ALICE);

    // enter Alice's 2000 ROTO for 1000 xROTO
    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Enter {}).unwrap(),
        amount: Uint128::from(2000u128),
    };

    router
        .execute_contract(
            alice_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Check if Alice's xROTO balance is 1000
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(1000u128)
        }
    );

    // Try to burn Alice's 2000 xROTO and unstake
    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Leave {}).unwrap(),
        amount: Uint128::from(2000u128),
    };

    let res = router
        .execute_contract(
            alice_address.clone(),
            x_roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap_err();

    assert_eq!(
        res.root_cause().to_string(),
        "Cannot Sub with 1000 and 2000"
    );
}

#[test]
fn should_work_with_more_than_one_participant() {
    let mut router = mock_app();

    let owner = Addr::unchecked("owner");

    let (roto_token_instance, staking_instance, x_roto_token_instance) =
        instantiate_contracts(&mut router, owner.clone());

    // Mint 10000 ROTO for Alice
    mint_some_roto(
        &mut router,
        owner.clone(),
        roto_token_instance.clone(),
        ALICE,
    );
    let alice_address = Addr::unchecked(ALICE);

    // Mint 10000 ROTO for Bob
    mint_some_roto(
        &mut router,
        owner.clone(),
        roto_token_instance.clone(),
        BOB,
    );
    let bob_address = Addr::unchecked(BOB);

    // Mint 10000 ROTO for Carol
    mint_some_roto(
        &mut router,
        owner.clone(),
        roto_token_instance.clone(),
        CAROL,
    );
    let carol_address = Addr::unchecked(CAROL);

    // Stake Alice's 2000 ROTO for 1000 xROTO (subtract min liquid amount)
    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Enter {}).unwrap(),
        amount: Uint128::from(2000u128),
    };

    router
        .execute_contract(
            alice_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Stake Bob's 10 ROTO for 10 xROTO
    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Enter {}).unwrap(),
        amount: Uint128::from(10u128),
    };

    router
        .execute_contract(bob_address.clone(), roto_token_instance.clone(), &msg, &[])
        .unwrap();

    // Check if Alice's xROTO balance is 1000
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(1000u128)
        }
    );

    // Check if Bob's xROTO balance is 10
    let msg = Cw20QueryMsg::Balance {
        address: bob_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(10u128)
        }
    );

    // Check if staking contract's ROTO balance is 2010
    let msg = Cw20QueryMsg::Balance {
        address: staking_instance.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(2010u128)
        }
    );

    // Staking contract gets 20 more ROTO from external source
    let msg = Cw20ExecuteMsg::Transfer {
        recipient: staking_instance.to_string(),
        amount: Uint128::from(20u128),
    };
    let res = router
        .execute_contract(
            carol_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap();
    assert_eq!(res.events[1].attributes[1], attr("action", "transfer"));
    assert_eq!(res.events[1].attributes[2], attr("from", carol_address));
    assert_eq!(
        res.events[1].attributes[3],
        attr("to", staking_instance.clone())
    );
    assert_eq!(
        res.events[1].attributes[4],
        attr("amount", Uint128::from(20u128))
    );

    // Stake Alice's 10 ROTO for 9 xROTO: 10*2010/2030 = 9
    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Enter {}).unwrap(),
        amount: Uint128::from(10u128),
    };

    router
        .execute_contract(
            alice_address.clone(),
            roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Check if Alice's xROTO balance is 1009
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(1009u128)
        }
    );

    // Check if Bob's xROTO balance is 10
    let msg = Cw20QueryMsg::Balance {
        address: bob_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(10u128)
        }
    );

    // Burn Bob's 5 xROTO and unstake: gets 5*2040/2019 = 5 ROTO
    let msg = Cw20ExecuteMsg::Send {
        contract: staking_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Leave {}).unwrap(),
        amount: Uint128::from(5u128),
    };

    router
        .execute_contract(
            bob_address.clone(),
            x_roto_token_instance.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Check if Alice's xROTO balance is 1009
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(1009u128)
        }
    );

    // Check if Bob's xROTO balance is 5
    let msg = Cw20QueryMsg::Balance {
        address: bob_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(5u128)
        }
    );

    // Check if the staking contract's ROTO balance is 52 (60 - 8 (Bob left 5 xROTO))
    let msg = Cw20QueryMsg::Balance {
        address: staking_instance.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(2035u128)
        }
    );

    // Check if Alice's ROTO balance is 7990 (10000 minted - 2000 entered - 10 entered)
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(7990u128)
        }
    );

    // Check if Bob's ROTO balance is 9995 (10000 minted - 10 entered + 5 by leaving)
    let msg = Cw20QueryMsg::Balance {
        address: bob_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: roto_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(9995u128)
        }
    );
}
