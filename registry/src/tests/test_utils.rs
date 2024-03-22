#![cfg(test)]

use crate::contract::{RegistryContract, RegistryContractClient};
use crate::storage::core::OffersConfig;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Bytes, Env, Vec};

fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(e, &contract_address),
        token::StellarAssetClient::new(e, &contract_address),
    )
}

pub struct TestData<'a> {
    pub contract_client: RegistryContractClient<'a>,

    pub adm: Address,
    pub node_rate: u128,
    pub col_asset: Address,
    pub col_asset_adm: Address,
    pub col_asset_client: token::Client<'a>,
    pub col_asset_stellar: token::StellarAssetClient<'a>,
    pub min_duration: u64,
    pub allowed_tlds: Vec<Bytes>,

    pub fee_taker: Address,
    pub offer_fee: u128,
}

pub fn create_test_data<'a>(e: &Env) -> TestData<'a> {
    let contract_id: Address = e.register_contract(None, RegistryContract);
    let contract_client: RegistryContractClient<'a> = RegistryContractClient::new(&e, &contract_id);

    let adm: Address = Address::generate(&e);
    let node_rate: u128 = 100;
    let col_asset_adm: Address = Address::generate(&e);
    let (col_asset_client, col_asset_stellar) = create_token_contract(&e, &col_asset_adm);

    let min_duration: u64 = 31536000;
    let allowed_tlds: Vec<Bytes> = Vec::from_array(
        &e,
        [
            Bytes::from_slice(&e, "xlm".as_bytes()),
            Bytes::from_slice(&e, "stellar".as_bytes()),
            Bytes::from_slice(&e, "wallet".as_bytes()),
            Bytes::from_slice(&e, "dao".as_bytes()),
        ],
    );

    let fee_taker: Address = Address::generate(&e);
    let offer_fee: u128 = 3_5000000;

    TestData {
        contract_client,
        adm,
        node_rate,
        col_asset: col_asset_client.address.clone(),
        col_asset_adm,
        col_asset_client,
        col_asset_stellar,
        min_duration,
        allowed_tlds,
        fee_taker,
        offer_fee,
    }
}

pub fn init_contract(test_data: &TestData) {
    test_data.contract_client.init(
        &test_data.adm,
        &test_data.node_rate,
        &test_data.col_asset,
        &test_data.min_duration,
        &test_data.allowed_tlds,
    );

    test_data
        .contract_client
        .mock_all_auths()
        .set_offers_config(&OffersConfig {
            fee_taker: test_data.fee_taker.clone(),
            fee: test_data.offer_fee.clone(),
        });
}
