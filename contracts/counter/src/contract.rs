#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Uint64, StdError,
};
use cw2::{set_contract_version, get_contract_version};
use cw20_base::contract::query_token_info;

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{State, STATE, A, ITEM_A, B, ITEM_B};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:my-first-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    STATE.save(deps.storage, &state)?;
    let a: A = A {
        name: String::default(),
        l_name: String::default(),
        age: Uint64::default(),
        num: Uint64::default(),
    };
    
    let b: B = B {
        name: String::default(),
        age: Uint64::default(),
    };
    ITEM_A.save(deps.storage, &a)?;
    ITEM_B.save(deps.storage, &b)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::Reset { count } => try_reset(deps, info, count),
    }
}

pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count = state.count.checked_add(Uint128::from(1u128)).unwrap();
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}
pub fn try_reset(
    deps: DepsMut,
    info: MessageInfo,
    count: Uint128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetContractVersion {} => to_binary(&get_contract_version(deps.storage)?),
        QueryMsg::GetA {} => to_binary(&get_a(deps)?),
        QueryMsg::GetB {} => to_binary(&get_b(deps)?)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let ver = cw2::get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME.to_string() {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

fn get_a(deps: Deps) -> StdResult<A> {
    let state = ITEM_A.load(deps.storage)?;
    Ok(state)
}

fn get_b(deps: Deps) -> StdResult<B> {
    let state = ITEM_B.load(deps.storage)?;
    Ok(state)
}