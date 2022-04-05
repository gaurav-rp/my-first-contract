pub use cw20_base::msg::InstantiateMsg;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, Uint128};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Set Coin's WhiteListing status
    WhiteListCoin { denom: String, status: bool },
    /// Deposit is a base messgae to deposit native coins in contract address and mint equivalent
    /// coins in user address
    Deposit {},
    /// Withdraw is a base messgae to withdraw native coins from contract address and burn equivalent
    /// coins from user address
    Withdraw {
        denom: String,
        amount: Uint128,
        recipient: Option<String>,
    },
    /// Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with the "mintable" extension. If authorized, creates amount new tokens
    /// and adds to the recipient balance.
    Mint { recipient: String, amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the current balance of the given address, 0 if unset.
    /// Return type: BalanceResponse.
    Balance { address: String},
    BalanceDenom { address: String, denom: String },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    /// Return type: TokenInfoResponse.
    TokenInfo {},
    /// Only with "mintable" extension.
    /// Returns who can mint and the hard cap on maximum tokens after minting.
    /// Return type: MinterResponse.
    Minter {},

}
