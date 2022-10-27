#[cfg(test)]
mod tests {
    use crate::msg::InstantiateMsg;
    use crate::state::MINTER_ADDR;
    use crate::{helpers::CwTemplateContract, msg::ExecuteMsg};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(MINTER_ADDR),
                    vec![Coin {
                        denom: "denom".to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let msg = InstantiateMsg {
            name: String::from("THONG COIN"),
            symbol: String::from("CUCMO"),
            cap: Uint128::from(300u128),
            total_supply: Uint128::from(200u128),
        };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(MINTER_ADDR),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    #[test]
    fn mint() {
        let (mut app, cw_template_contract) = proper_instantiate();

        let msg = ExecuteMsg::Mint {
            recipient: (&MINTER_ADDR).to_string(),
            amount: Uint128::from(100u128),
        };
        let cosmos_msg = cw_template_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(MINTER_ADDR), cosmos_msg)
            .unwrap();
    }
}
