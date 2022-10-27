use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{StdResult, Uint128};

#[cw_serde]
#[cfg_attr(test, derive(Default))]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub cap: Uint128,
    pub total_supply: Uint128,
}

impl InstantiateMsg {
    pub fn validate(&self) -> StdResult<()> {
        // TODO: implement
        Ok(())
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    Transfer { recipient: String, amount: Uint128 },
    Burn { amount: Uint128 },
    Mint { recipient: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    Balance { address: String },
}
