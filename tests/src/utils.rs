use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, token};

pub fn create_token_contract<'a>(e: &Env, admin: &Address) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(e, &contract_address),
        token::StellarAssetClient::new(e, &contract_address),
    )
}

pub struct GlobalTestData<'a> {
    pub col_asset: Address,
    pub col_asset_adm: Address,
    pub col_asset_client: token::Client<'a>,
    pub col_asset_stellar: token::StellarAssetClient<'a>,

    pub gov_asset: Address,
    pub gov_asset_adm: Address,
    pub gov_asset_client: token::Client<'a>,
    pub gov_asset_stellar: token::StellarAssetClient<'a>,

    pub registry_v1: Address,
    pub registry_v2: Address,
    pub nfd: Address,
    pub oracle_addr: Address,

    pub adm: Address,
    pub fee_taker: Address,
}

pub fn create_global_test_data<'a>(e: &Env) -> GlobalTestData<'a> {
    let adm: Address = Address::generate(&e);
    let col_asset_adm: Address = Address::generate(&e);
    let (col_asset_client, col_asset_stellar) = create_token_contract(&e, &col_asset_adm);
    let gov_asset_adm: Address = Address::generate(&e);
    let (gov_asset_client, gov_asset_stellar) = create_token_contract(&e, &gov_asset_adm);
    let fee_taker: Address = Address::generate(&e);

    let registry_v1: Address = Address::generate(&e);
    let registry_v2: Address = Address::generate(&e);
    let nfd: Address = Address::generate(&e);

    let oracle_addr: Address = Address::from_string(&soroban_sdk::String::from_str(
        e,
        "CAFJZQWSED6YAWZU3GWRTOCNPPCGBN32L7QV43XX5LZLFTK6JLN34DLN",
    ));

    GlobalTestData {
        col_asset: col_asset_client.address.clone(),
        col_asset_adm,
        col_asset_client,
        col_asset_stellar,

        gov_asset: gov_asset_client.address.clone(),
        gov_asset_adm,
        gov_asset_client,
        gov_asset_stellar,

        registry_v1,
        registry_v2,
        nfd,
        oracle_addr,

        adm,
        fee_taker,
    }
}
