pub mod import {
    soroban_sdk::contractimport!(file = "../oracle.wasm");
}

use import::Client as PriceOracleContractClient;
use import::{Asset, ConfigData};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, symbol_short, vec};

pub fn init_reflector<'a>(e: &Env, admin: &Address, contract_id: &Address) -> (PriceOracleContractClient<'a>, ConfigData) {
    e.register_at(&contract_id, import::WASM, ());
    let client: PriceOracleContractClient<'a> = PriceOracleContractClient::new(&e, contract_id);

    let init_data = ConfigData {
        admin: admin.clone(),
        period: 100 * 300_000,
        assets: vec![&e, Asset::Other(symbol_short!("XLM"))],
        base_asset: Asset::Stellar(Address::generate(&e)),
        decimals: 14,
        resolution: 300_000,
    };

    // set admin
    client.mock_all_auths().config(&init_data);

    (client, init_data)
}
