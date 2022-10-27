# Minimal implementation for CW20 Token (CosmWasm)

Minimal implementation of CW20 token on CosmWasm.
Supported functionalities:
* Transfer tokens
* Mint tokens (from only fixed address)
* Burn tokens

## Guide
This guideline is based on [official CosmWasm doc](https://docs.cosmwasm.com/docs/1.0/).

### Prepare
1. Requirement: [Step-by-step](https://docs.cosmwasm.com/docs/1.0/getting-started/installation)
    * `wasmd`
    * `Rust` with wasm32 build target 
    * `jq` (tool to parse json response when execute smart contract)
2. Create your wallet (should create more than 1 wallet for testing transfer)
    ```bash
    wasmd keys add {your-wallet-name}
    ```
3. Export those variables to use later
    ```bash
    # We gonna use malaga testnet
    export CHAIN_ID="malaga-420"  
    export TESTNET_NAME="malaga-420"
    export RPC="https://rpc.malaga-420.cosmwasm.com:443"
    export API="https://api.malaga-420.cosmwasm.com"
    export FAUCET="https://faucet.malaga-420.cosmwasm.com"

    export FEE_DENOM="umlg"
    export STAKE_DENOM="uand"
    export BECH32_HRP="wasm"
    export WASMD_VERSION="v0.27.0"
    export CONFIG_DIR=".wasmd"
    export BINARY="wasmd"

    export NODE=(--node $RPC)
    export TXFLAG=($NODE --chain-id $CHAIN_ID --gas-prices 0.25$FEE_DENOM --gas auto --gas-adjustment 1.3)
    ```
4. Add fund to your wallet (to deploy and run smart contract, called `gas` in Ethereuum)
    ```bash
    JSON=$(jq -n --arg addr $(wasmd keys show -a {your-wallet-name}) '{"denom":"umlg","address":$addr}') && curl -X POST --header "Content-Type: application/json" --data "$JSON" https://faucet.malaga-420.cosmwasm.com/credit

    # Quick check your balance
    wasmd query bank balances $(wasmd keys show -a {your-wallet-name}) $NODE
    ```

### Compile & Deploy & Interact
1. Update minter address in `src/state.rs` to yours
2. Compile to `.wasm` file (You can also use docker for more optimized [here](https://docs.cosmwasm.com/docs/1.0/getting-started/compile-contract#optimized-compilation))
    ```bash
    RUSTFLAGS='-C link-arg=-s' cargo wasm
    ```
3. Deploy
    ```bash
    # Upload your smart contract
    RES=$(wasmd tx wasm store {your-path-to-wasm-file} --from wallet $TXFLAG -y --output json -b block)

    # Get code_id of your smart contract
    CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
    ```
4. Initialize
    ```bash
    # InstantiateMsg (change this on your preference)
    INIT_MSG='{"name": "THONG COIN","symbol": "CUCMO","cap": "1000000","total_supply": "500000"}'

    # Initialize contract with InstantiateMsg
    wasmd tx wasm instantiate $CODE_ID "$INIT_MSG" --from {your-wallet-name} --label {your-contract-label} $TXFLAG -y --no-admin

    # Check list of contract that initialized before
    wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json

    # Extract the contract address to use later
    CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')

    ```
5. Interact
    ```bash
    # Wallet name that have tokens
    WALLET_FROM={your-wallet-name}

    # Wallet name that you want to transfer to
    WALLET_TO={another-wallet-name}

    # TransferMsg (change this on your preference)
    TRANSFER='{"transfer":{"recipient":"'$(wasmd keys show -a $WALLET_TO)'","amount":"55"}}'
    echo $TRANSFER

    # Execute TRANSFER command on smart contract
    wasmd tx wasm execute $CONTRACT "$TRANSFER" --from $WALLET_FROM $TXFLAG -y

    # QueryBalanceMsg (change this on your preference)
    BALANCE='{"balance": {"address": "'$(wasmd keys show -a $WALLET_FROM)'"}}'

    # Execute QUERY balance command on smart contract
    wasmd query wasm contract-state smart $CONTRACT "$BALANCE" $NODE --output json
    ```
