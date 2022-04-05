use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

pub const WHITELISTED_COINS: Map<&str, bool> = Map::new("whitelisted_coins");

pub const BALANCES: Map<(&Addr, &str), Uint128> = Map::new("balances");
