use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, BytesN, Env, Vec};

pub mod import {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32v1-none/release/registry.wasm");
}

use crate::utils::GlobalTestData;
use import::{Client as RegistryV1Client, WASM as RegistryV1};

pub struct RegistryV1TestData<'a> {
    pub contract_client: RegistryV1Client<'a>,
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

impl RegistryV1TestData<'_> {
    pub fn create_test_data<'a>(e: &Env, global_test_data: &GlobalTestData) -> RegistryV1TestData<'a> {
        let contract_id: Address = e.register_at(&global_test_data.registry_v1, RegistryV1, ());
        let contract_client: RegistryV1Client<'a> = RegistryV1Client::new(&e, &contract_id);

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

        RegistryV1TestData {
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

    pub fn init_contract(global_test_data: &GlobalTestData, test_data: &RegistryV1TestData) {
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
