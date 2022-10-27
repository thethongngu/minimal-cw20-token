use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct TokenConfig {
    pub name: String,
    pub symbol: String,
    pub total_supply: Uint128,
    pub cap: Uint128,
}

pub const TOKEN_CONFIG: Item<TokenConfig> = Item::new("token_info");
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");
pub const MINTER_ADDR: &str = "wasm1002p8x7ajerzwnvjgguqeymplh70y8shs6d0hy";
