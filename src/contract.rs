#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{TokenConfig, BALANCES, MINTER_ADDR, TOKEN_CONFIG};

use self::execute::{execute_burn, execute_mint, execute_transfer};
use self::query::query_balance;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:thong-coin-v1";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.total_supply > msg.cap {
        return Err(ContractError::SupplyExceedCap {});
    }

    let token_config = TokenConfig {
        name: msg.name,
        symbol: msg.symbol,
        total_supply: msg.total_supply,
        cap: msg.cap,
    };
    TOKEN_CONFIG.save(deps.storage, &token_config)?;
    let minter = deps.api.addr_validate(&MINTER_ADDR)?;
    BALANCES.save(deps.storage, &minter, &msg.total_supply)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("minter", MINTER_ADDR))
}

pub mod execute {
    use super::*;
    use crate::state::BALANCES;
    use cosmwasm_std::Uint128;

    pub fn execute_transfer(
        mut deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }

        let recipient = deps.api.addr_validate(&recipient)?;
        sync_up_new_address(&mut deps, &info.sender)?;
        sync_up_new_address(&mut deps, &recipient)?;

        BALANCES.update(deps.storage, &info.sender, |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        })?;
        BALANCES.update(deps.storage, &recipient, |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_add(amount)?)
        })?;

        Ok(Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("from", info.sender)
            .add_attribute("to", recipient)
            .add_attribute("amount", amount))
    }

    pub fn execute_burn(
        mut deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }
        sync_up_new_address(&mut deps, &info.sender)?;

        BALANCES.update(deps.storage, &info.sender, |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        })?;
        TOKEN_CONFIG.update(deps.storage, |config| -> StdResult<_> {
            config.total_supply.checked_sub(amount)?;
            Ok(config)
        })?;

        Ok(Response::new()
            .add_attribute("action", "burn")
            .add_attribute("from", info.sender)
            .add_attribute("amount", amount))
    }

    pub fn execute_mint(
        mut deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }

        // TODO: other contract doesn't have TOKEN_CONFIG?
        let mut config = TOKEN_CONFIG
            .may_load(deps.storage)?
            .ok_or(ContractError::Unauthorized {})?;

        if MINTER_ADDR != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if config.total_supply + amount > config.cap {
            return Err(ContractError::CannotExceedCap {});
        }

        config.total_supply += amount;
        TOKEN_CONFIG.save(deps.storage, &config)?;

        let recipient = deps.api.addr_validate(&recipient)?;
        sync_up_new_address(&mut deps, &recipient)?;
        BALANCES.update(deps.storage, &recipient, |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_add(amount)?)
        })?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("to", recipient)
            .add_attribute("amount", amount))
    }

    fn sync_up_new_address(deps: &mut DepsMut, addr: &Addr) -> StdResult<()> {
        if BALANCES.has(deps.storage, addr) {
            return Ok(());
        }
        BALANCES.save(deps.storage, addr, &Uint128::zero())
    }
}

pub mod query {
    use super::*;
    use crate::state::BALANCES;
    use cosmwasm_std::Uint128;

    pub fn query_balance(deps: Deps, address: String) -> StdResult<Uint128> {
        let address = deps.api.addr_validate(&address)?;
        let balance = BALANCES
            .may_load(deps.storage, &address)?
            .unwrap_or_default();
        Ok(balance)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::Mint { recipient, amount } => execute_mint(deps, env, info, recipient, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
    }
}

#[cfg(test)]
mod tests {
    use crate::msg;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Uint128};

    const RECIPIENT: &str = "wasm1exntxrfdafgczcuwhuuqr5pftkln3w0d6cwnrv";
    const MINTER: &str = "wasm1002p8x7ajerzwnvjgguqeymplh70y8shs6d0hy";

    fn get_init_msg() -> msg::InstantiateMsg {
        InstantiateMsg {
            name: String::from("THONG COIN"),
            symbol: String::from("CUCMO"),
            cap: Uint128::from(300u128),
            total_supply: Uint128::from(200u128),
        }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, get_init_msg()).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Balance {
                address: MINTER_ADDR.to_string(),
            },
        )
        .unwrap();

        let value: Uint128 = from_binary(&res).unwrap();
        assert_eq!(value, Uint128::from(200u128));
    }

    #[test]
    fn transfer() {
        // initiate contract with minter address has 200 CUCMO
        let mut deps = mock_dependencies();
        let info = mock_info(MINTER, &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), get_init_msg()).unwrap();

        // verify current balance in minter
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Balance {
                address: MINTER.to_string(),
            },
        )
        .unwrap();
        let value: Uint128 = from_binary(&res).unwrap();
        assert_eq!(value, Uint128::from(200u128));

        // verify transfer
        let transfer_msg = ExecuteMsg::Transfer {
            recipient: RECIPIENT.to_string(),
            amount: Uint128::from(150u128),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, transfer_msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Balance {
                address: RECIPIENT.to_string(),
            },
        )
        .unwrap();
        let value: Uint128 = from_binary(&res).unwrap();
        assert_eq!(value, Uint128::from(150u128));

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Balance {
                address: MINTER.to_string(),
            },
        )
        .unwrap();
        let value: Uint128 = from_binary(&res).unwrap();
        assert_eq!(value, Uint128::from(50u128));
    }

    #[test]
    fn burn() {
        let mut deps = mock_dependencies();
        let info = mock_info(MINTER, &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), get_init_msg()).unwrap();

        // verify burn
        let transfer_msg = ExecuteMsg::Burn {
            amount: Uint128::from(250u128),
        };
        assert!(execute(deps.as_mut(), mock_env(), info.clone(), transfer_msg).is_err());

        let transfer_msg = ExecuteMsg::Burn {
            amount: Uint128::from(200u128),
        };
        assert!(execute(deps.as_mut(), mock_env(), info, transfer_msg).is_ok());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Balance {
                address: MINTER.to_string(),
            },
        )
        .unwrap();
        let value: Uint128 = from_binary(&res).unwrap();
        assert_eq!(value, Uint128::from(0u128));
    }
}
