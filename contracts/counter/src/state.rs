use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Uint64};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: Uint128,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct A {
    pub name: String,
    pub l_name: String,
    pub age: Uint64,
    pub num: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct B {
    pub name: String,
    pub age: Uint64,
}

pub const STATE: Item<State> = Item::new("state");
pub const ITEM_A: Item<A> = Item::new("a");
pub const ITEM_B: Item<B> = Item::new("b");
