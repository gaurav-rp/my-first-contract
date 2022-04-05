use crate::state::{BALANCES, WHITELISTED_COINS};

#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cosmwasm_std::{to_binary, BankMsg, Coin, CosmosMsg, StdError, Uint128, Addr};
use cw2::set_contract_version;
use cw20::BalanceResponse;
use cw20_base::{
    contract::{
        execute_mint, execute_send, execute_transfer, instantiate as cw_instantiate,
        query_minter, query_token_info, query_balance,
    },
    ContractError,
};
use crate::erc20::{ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "erc-20";
const CONTRACT_VERSION: &str = "1.0.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw_instantiate(deps.branch(), env.clone(), info.clone(), msg)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "erc20_contract_intantiated"))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::WhiteListCoin { denom, status } => {
            set_coin_white_listing(deps, info, denom, status)
        }
        ExecuteMsg::Deposit {} => deposit(deps, env, info),
        ExecuteMsg::Withdraw {
            denom,
            amount,
            recipient,
        } => withdraw(deps, env, info, denom, amount, recipient),
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Mint { recipient, amount } => execute_mint(deps, env, info, recipient, amount)
    }
}

fn set_coin_white_listing(
    deps: DepsMut,
    _info: MessageInfo,
    denom: String,
    status: bool,
) -> Result<Response, ContractError> {
    WHITELISTED_COINS.save(deps.storage, &denom, &status)?;
    Ok(Response::default())
}

fn deposit(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let recipient: String = info.sender.to_string();
    let coins: Vec<Coin> = info.funds.clone();

    if coins.len() != 1 {
        return Err(ContractError::Std(StdError::generic_err(
            "incorrect funds are sent",
        )));
    }
    let denom: &str = &coins[0].denom;
    let amount: Uint128 = coins[0].amount;
    is_white_listed_denom(deps.as_ref(), &coins[0].denom)?;

    
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // add amount to recipient balance
    BALANCES.update(
        deps.storage,
        (&info.sender, denom),
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("to", recipient)
        .add_attribute("denom", denom.to_string())
        .add_attribute("amount", amount);
    Ok(res)
}

fn withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: String,
    amount: Uint128,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    is_white_listed_denom(deps.as_ref(), &denom)?;
    let receiver: Addr = match recipient {
        Some(address) => {
            deps.api.addr_validate(&address)?
        },
        None => info.sender.clone(),
    };
    
    let exec_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: receiver.to_string(),
        amount: vec![Coin {
            denom: denom.clone(),
            amount: amount,
        }],
    });
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // lower balance
    BALANCES.update(
        deps.storage,
        (&receiver, &denom),
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;

    let res = Response::new()
        .add_attribute("action", "withdraw")
        .add_message(exec_msg)
        .add_attribute("from", receiver.to_string())
        .add_attribute("denom", denom)
        .add_attribute("amount", amount);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::BalanceDenom { address, denom } => to_binary(&query_balance_info(deps, address, denom)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
    }
}

fn query_balance_info(deps: Deps, address: String, denom: String)-> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCES
        .may_load(deps.storage, (&address, &denom))?
        .unwrap_or_default();
    Ok(BalanceResponse { balance })
}

fn is_white_listed_denom(deps: Deps, denom: &str) -> Result<bool, ContractError> {
    let is_white_listed_coin: bool = match WHITELISTED_COINS.load(deps.storage, denom) {
        Ok(bool_flag) => bool_flag,
        Err(_) => false,
    };
    if is_white_listed_coin {
        return Ok(is_white_listed_coin);
    }
    return Err(ContractError::Std(StdError::generic_err(
        "the coin is not whitelisted",
    )));
}
