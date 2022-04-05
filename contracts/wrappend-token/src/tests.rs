use crate::contract::{execute, instantiate};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    to_binary, Binary, Coin, Deps, DepsMut, StdError, Uint128,
};
use cw2::{get_contract_version, ContractVersion};
use cw20::{BalanceResponse, Cw20Coin, MinterResponse, TokenInfoResponse};
use cw20_base::{
    contract::{query_balance, query_minter, query_token_info},
    ContractError,
};
use crate::erc20::{ExecuteMsg, InstantiateMsg};

const INIT_ADDRESS: &str = "contract_initiator";
const RECIPIENT: &str = "recipient";
const MINTER: &str = "minter";

fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
    query_balance(deps, address.into()).unwrap().balance
}

// this will set up the instantiation for other tests
fn do_instantiate_with_minter(
    deps: DepsMut,
    addr: &str,
    amount: Uint128,
    minter: &str,
    cap: Option<Uint128>,
) -> TokenInfoResponse {
    _do_instantiate(
        deps,
        addr,
        amount,
        Some(MinterResponse {
            minter: minter.to_string(),
            cap,
        }),
    )
}

// this will set up the instantiation for other tests
fn do_instantiate(deps: DepsMut, addr: &str, amount: Uint128) -> TokenInfoResponse {
    _do_instantiate(deps, addr, amount, None)
}

// this will set up the instantiation for other tests
fn _do_instantiate(
    mut deps: DepsMut,
    addr: &str,
    amount: Uint128,
    mint: Option<MinterResponse>,
) -> TokenInfoResponse {
    let instantiate_msg = InstantiateMsg {
        name: "Router Protocol".to_string(),
        symbol: "ROUTE".to_string(),
        decimals: 18,
        initial_balances: vec![Cw20Coin {
            address: addr.to_string(),
            amount,
        }],
        mint: mint.clone(),
        marketing: None,
    };
    let info = mock_info(INIT_ADDRESS, &[]);
    let env = mock_env();
    let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let meta = query_token_info(deps.as_ref()).unwrap();
    assert_eq!(
        meta,
        TokenInfoResponse {
            name: "Router Protocol".to_string(),
            symbol: "ROUTE".to_string(),
            decimals: 18,
            total_supply: amount,
        }
    );
    assert_eq!(get_balance(deps.as_ref(), addr), amount);
    assert_eq!(query_minter(deps.as_ref()).unwrap(), mint);
    let version_info: ContractVersion = ContractVersion {
        contract: String::from("erc-20"),
        version: String::from("1.0.0"),
    };
    assert_eq!(version_info, get_contract_version(deps.storage).unwrap());

    meta
}

#[test]
fn test_basic() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount = Uint128::from(11223344u128);
    let instantiate_msg = InstantiateMsg {
        name: "Router Protocol".to_string(),
        symbol: "ROUTE".to_string(),
        decimals: 9,
        initial_balances: vec![Cw20Coin {
            address: String::from("addr0000"),
            amount,
        }],
        mint: None,
        marketing: None,
    };
    let info = mock_info(INIT_ADDRESS, &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(
        query_token_info(deps.as_ref()).unwrap(),
        TokenInfoResponse {
            name: "Router Protocol".to_string(),
            symbol: "ROUTE".to_string(),
            decimals: 9,
            total_supply: amount,
        }
    );
    assert_eq!(
        get_balance(deps.as_ref(), "addr0000"),
        Uint128::new(11223344)
    );
}

#[test]
fn test_transfer() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount = Uint128::new(100);
    do_instantiate(deps.as_mut(), RECIPIENT, amount);

    let info = mock_info(RECIPIENT, &[]);
    let env = mock_env();

    let winner = String::from("lucky");
    let prize = Uint128::new(60);
    let msg = ExecuteMsg::Transfer {
        recipient: winner.clone(),
        amount: prize,
    };

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    assert_eq!(0, res.messages.len());
    assert_eq!(get_balance(deps.as_ref(), RECIPIENT), Uint128::new(40));
    assert_eq!(get_balance(deps.as_ref(), &winner), Uint128::new(60));

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(res.is_err(), true);
    let err = res.unwrap_err();
    assert_eq!(
        err.to_string(),
        String::from("Overflow: Cannot Sub with 40 and 60")
    );
}

#[test]
fn test_send() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount = Uint128::new(100);
    do_instantiate(deps.as_mut(), RECIPIENT, amount);

    let info = mock_info(RECIPIENT, &[]);
    let env = mock_env();

    let winner = String::from("lucky");
    let prize = Uint128::new(60);
    let binary_data: Binary = to_binary(&{}).unwrap();
    let msg = ExecuteMsg::Send {
        contract: winner.clone(),
        amount: prize,
        msg: binary_data,
    };

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(res.is_ok(), true);

    assert_eq!(get_balance(deps.as_ref(), RECIPIENT), Uint128::new(40));
    assert_eq!(get_balance(deps.as_ref(), &winner), Uint128::new(60));

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert_eq!(res.is_err(), true);
    let err = res.unwrap_err();
    assert_eq!(
        err.to_string(),
        String::from("Overflow: Cannot Sub with 40 and 60")
    );
}

#[test]
fn mintable() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount = Uint128::new(11223344);
    let limit = Uint128::new(511223344);
    do_instantiate_with_minter(deps.as_mut(), RECIPIENT, amount, MINTER, Some(limit));

    assert_eq!(
        query_token_info(deps.as_ref()).unwrap(),
        TokenInfoResponse {
            name: "Router Protocol".to_string(),
            symbol: "ROUTE".to_string(),
            decimals: 18,
            total_supply: amount,
        }
    );
    assert_eq!(
        get_balance(deps.as_ref(), RECIPIENT),
        Uint128::new(11223344)
    );
    assert_eq!(
        query_minter(deps.as_ref()).unwrap(),
        Some(MinterResponse {
            minter: String::from(MINTER),
            cap: Some(limit),
        }),
    );
}

#[test]
fn mintable_over_cap() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount = Uint128::new(11223344);
    let minter = String::from("asmodat");
    let limit = Uint128::new(11223300);
    let instantiate_msg = InstantiateMsg {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 9,
        initial_balances: vec![Cw20Coin {
            address: String::from("addr0000"),
            amount,
        }],
        mint: Some(MinterResponse {
            minter,
            cap: Some(limit),
        }),
        marketing: None,
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let err = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Initial supply greater than cap").into()
    );
}

#[test]
fn can_mint_by_minter() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);

    let genesis = String::from("genesis");
    let amount = Uint128::new(11223344);
    let minter = String::from("asmodat");
    let limit = Uint128::new(511223344);
    do_instantiate_with_minter(deps.as_mut(), &genesis, amount, &minter, Some(limit));

    // minter can mint coins to some winner
    let winner = String::from("lucky");
    let prize = Uint128::new(222_222_222);
    let msg = ExecuteMsg::Mint {
        recipient: winner.clone(),
        amount: prize,
    };

    let info = mock_info(minter.as_ref(), &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(get_balance(deps.as_ref(), genesis), amount);
    assert_eq!(get_balance(deps.as_ref(), winner.clone()), prize);

    // but cannot mint nothing
    let msg = ExecuteMsg::Mint {
        recipient: winner.clone(),
        amount: Uint128::zero(),
    };
    let info = mock_info(minter.as_ref(), &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // but if it exceeds cap (even over multiple rounds), it fails
    // cap is enforced
    let msg = ExecuteMsg::Mint {
        recipient: winner,
        amount: Uint128::new(333_222_222),
    };
    let info = mock_info(minter.as_ref(), &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::CannotExceedCap {});
}

#[test]
fn others_cannot_mint() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    do_instantiate_with_minter(
        deps.as_mut(),
        &String::from("genesis"),
        Uint128::new(1234),
        &String::from("minter"),
        None,
    );

    let msg = ExecuteMsg::Mint {
        recipient: String::from("lucky"),
        amount: Uint128::new(222),
    };
    let info = mock_info("anyone else", &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn no_one_mints_if_minter_unset() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    do_instantiate(deps.as_mut(), &String::from("genesis"), Uint128::new(1234));

    let msg = ExecuteMsg::Mint {
        recipient: String::from("lucky"),
        amount: Uint128::new(222),
    };
    let info = mock_info("genesis", &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn instantiate_multiple_accounts() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount1 = Uint128::from(11223344u128);
    let addr1 = String::from("addr0001");
    let amount2 = Uint128::from(7890987u128);
    let addr2 = String::from("addr0002");
    let instantiate_msg = InstantiateMsg {
        name: "Bash Shell".to_string(),
        symbol: "BASH".to_string(),
        decimals: 6,
        initial_balances: vec![
            Cw20Coin {
                address: addr1.clone(),
                amount: amount1,
            },
            Cw20Coin {
                address: addr2.clone(),
                amount: amount2,
            },
        ],
        mint: None,
        marketing: None,
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    assert_eq!(
        query_token_info(deps.as_ref()).unwrap(),
        TokenInfoResponse {
            name: "Bash Shell".to_string(),
            symbol: "BASH".to_string(),
            decimals: 6,
            total_supply: amount1 + amount2,
        }
    );
    assert_eq!(get_balance(deps.as_ref(), addr1), amount1);
    assert_eq!(get_balance(deps.as_ref(), addr2), amount2);
}

#[test]
fn queries_work() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::new(2),
        denom: String::from("token"),
    }]);
    let addr1 = String::from("addr0001");
    let amount1 = Uint128::from(12340000u128);

    let expected = do_instantiate(deps.as_mut(), &addr1, amount1);

    // check meta query
    let loaded = query_token_info(deps.as_ref()).unwrap();
    assert_eq!(expected, loaded);

    let loaded: BalanceResponse = query_balance(deps.as_ref(), addr1).unwrap();
    assert_eq!(loaded.balance, amount1);

    // check balance query (empty)
    let loaded: BalanceResponse = query_balance(deps.as_ref(), String::from("addr0002")).unwrap();

    assert_eq!(loaded.balance, Uint128::zero());
}
