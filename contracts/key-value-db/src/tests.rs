#![cfg(test)]

use soroban_sdk::Env;
use test_utils::{create_env, create_global_test_data, key_value_db, registry, GlobalTestData};

#[test]
fn test_keys() {
    let e: Env = create_env();

    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let key_value_db_test_data: key_value_db::TestData = key_value_db::create_test_data(&e);

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&registry_test_data.test_domain_owner, &i128::MAX);

    global_test_data
        .gov_asset_stellar
        .mock_all_auths()
        .mint(&registry_test_data.test_domain_owner, &i128::MAX);

    registry::init_contract(&global_test_data, &registry_test_data);
    key_value_db::init_contract(
        &global_test_data,
        &registry_test_data,
        &key_value_db_test_data,
    );

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(
            &registry_test_data.test_domain,
            &registry_test_data.test_tld,
            &registry_test_data.test_domain_owner,
            &registry_test_data.test_domain_owner,
            &registry_test_data.min_duration,
        );

    key_value_db_test_data.contract_client.mock_all_auths().set(
        &registry_test_data.test_node,
        &key_value_db_test_data.test_key,
        &key_value_db_test_data.test_value,
    );

    let saved_value = key_value_db_test_data
        .contract_client
        .get(
            &registry_test_data.test_node,
            &key_value_db_test_data.test_key,
        )
        .unwrap();

    assert_eq!(saved_value, key_value_db_test_data.test_value);
}

// TODO: test cases where no signature is provided, snapshot changed, providing wrong Record node and not enough funds to pay fee
