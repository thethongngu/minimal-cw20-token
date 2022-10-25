use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct TokenConfig {
    pub name: String,
    pub symbol: String,
    pub total_supply: Uint128,
    pub mint: Option<MinterData>,
}

#[cw_serde]
pub struct MinterData {
    pub minter: Addr,
    pub cap: Option<Uint128>, // in case of the token is not initialized yet?
}

impl TokenConfig {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }
}

pub const TOKEN_INFO: Item<TokenConfig> = Item::new("token_info");
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");
