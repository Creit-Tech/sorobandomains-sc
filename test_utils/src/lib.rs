use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{token, Address, Env};

pub fn create_token_contract<'a>(e: &Env, admin: &Address) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(e, &contract_address),
        token::StellarAssetClient::new(e, &contract_address),
    )
}

pub mod registry_contract {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/registry.wasm");
}

pub fn create_env() -> Env {
    let e: Env = Env::from_ledger_snapshot_file("../../network_snapshots/reflector.json");
    e.ledger().set_protocol_version(21);
    e.ledger().set_timestamp(1742825701);
    e
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

pub mod registry {
    use crate::{registry_contract, GlobalTestData};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, BytesN, Env, Vec};

    pub struct TestData<'a> {
        pub contract_client: registry_contract::Client<'a>,
        pub node_rate: u128,
        pub min_duration: u64,
        pub allowed_tlds: Vec<Bytes>,
        pub offer_fee: u128,

        pub test_domain_owner: Address,
        pub test_domain_target: Address,
        pub test_domain: Bytes,
        pub test_tld: Bytes,
        pub test_node: BytesN<32>,
    }

    pub fn create_test_data<'a>(e: &Env, global_test_data: &GlobalTestData) -> TestData<'a> {
        let contract_id: Address = e.register_at(&global_test_data.registry_v1, registry_contract::WASM, ());
        let contract_client: registry_contract::Client<'a> = registry_contract::Client::new(&e, &contract_id);

        let node_rate: u128 = 100;

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

        let offer_fee: u128 = 3_5000000;

        let test_domain_owner: Address = Address::generate(&e);
        let test_domain_target: Address = Address::generate(&e);
        let test_domain: Bytes = Bytes::from_slice(&e, "stellar".as_bytes());
        let test_tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
        let node_bytes: [u8; 32] = [
            47, 228, 204, 106, 21, 249, 70, 107, 173, 113, 237, 64, 122, 143, 27, 125, 168, 30, 253, 147, 30, 119, 18, 117, 49, 82, 170,
            23, 171, 192, 224, 110,
        ];

        TestData {
            contract_client,
            node_rate,
            min_duration,
            allowed_tlds,
            offer_fee,
            test_domain_owner,
            test_domain_target,
            test_domain,
            test_tld,
            test_node: BytesN::from_array(&e, &node_bytes),
        }
    }

    pub fn init_contract(global_test_data: &GlobalTestData, test_data: &TestData) {
        test_data.contract_client.init(
            &global_test_data.adm,
            &test_data.node_rate,
            &global_test_data.col_asset,
            &test_data.min_duration,
            &test_data.allowed_tlds,
        );

        test_data
            .contract_client
            .mock_all_auths()
            .set_offers_config(&global_test_data.fee_taker, &test_data.offer_fee);

        test_data.contract_client.mock_all_auths().set_oracle(&global_test_data.oracle_addr);
    }
}

pub mod key_value_db_contract {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/key_value_db.wasm");
}

pub mod key_value_db {
    use crate::{key_value_db_contract, registry, GlobalTestData};
    use soroban_sdk::{symbol_short, Address, Env, String, Symbol};

    pub struct TestData<'a> {
        pub contract_client: key_value_db_contract::Client<'a>,
        pub fee: u128,
        pub test_key: Symbol,
        pub test_value: key_value_db_contract::Value,
    }

    pub fn create_test_data<'a>(e: &Env) -> TestData<'a> {
        let contract_id: Address = e.register_contract_wasm(None, key_value_db_contract::WASM);
        let contract_client: key_value_db_contract::Client<'a> = key_value_db_contract::Client::new(&e, &contract_id);

        let test_key: Symbol = symbol_short!("Hello");
        let test_value: key_value_db_contract::Value = key_value_db_contract::Value::String(String::from_str(&e, "World!"));

        let fee: u128 = 100_0000000u128;

        TestData {
            contract_client,
            fee,
            test_key,
            test_value,
        }
    }

    pub fn init_contract(global_test_data: &GlobalTestData, registry_test_data: &registry::TestData, test_data: &TestData) {
        test_data.contract_client.set_config(
            &global_test_data.adm,
            &registry_test_data.contract_client.address,
            &test_data.fee,
            &global_test_data.gov_asset,
            &global_test_data.fee_taker,
        );
    }
}

pub mod reverse_registrar_contract {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/reverse_registrar.wasm");
}

pub mod reverse_registrar {
    use crate::{registry, reverse_registrar_contract, GlobalTestData};
    use soroban_sdk::{Address, Env};

    pub struct TestData<'a> {
        pub contract_client: reverse_registrar_contract::Client<'a>,
        pub fee: i128,
    }

    pub fn create_test_data<'a>(e: &Env) -> TestData<'a> {
        let contract_id: Address = e.register_contract_wasm(None, reverse_registrar_contract::WASM);
        let contract_client: reverse_registrar_contract::Client<'a> = reverse_registrar_contract::Client::new(&e, &contract_id);
        TestData {
            contract_client,
            fee: 100_0000000,
        }
    }

    pub fn init_contract(global_test_data: &GlobalTestData, registry_test_data: &registry::TestData, test_data: &TestData) {
        test_data.contract_client.set_config(
            &global_test_data.adm,
            &registry_test_data.contract_client.address,
            &test_data.fee,
            &global_test_data.gov_asset,
            &global_test_data.fee_taker,
        );
    }
}

pub mod reflector_contract {
    soroban_sdk::contractimport!(file = "../oracle.wasm");
}

pub mod reflector {
    use crate::reflector_contract;
    use crate::reflector_contract::Client as PriceOracleContractClient;
    use crate::reflector_contract::{Asset, ConfigData};
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{symbol_short, vec, Address, Env};

    pub fn init_reflector<'a>(e: &Env, admin: &Address, contract_id: &Address) -> (PriceOracleContractClient<'a>, ConfigData) {
        //set timestamp to 900 seconds
        let ledger_info = e.ledger().get();
        e.ledger().set(LedgerInfo {
            timestamp: 900,
            ..ledger_info
        });

        e.register_at(&contract_id, reflector_contract::WASM, ());
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
}
