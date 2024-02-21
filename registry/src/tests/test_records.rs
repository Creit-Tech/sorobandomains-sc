#![cfg(test)]

use crate::errors::ContractErrors;
use crate::storage::record::Record;
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, BytesN, Env};

#[test]
fn test_setting_record() {
    let e: Env = Env::default();
    e.mock_all_auths();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let new_owner: Address = Address::generate(&e);
    let new_address_target: Address = Address::generate(&e);
    let new_domain: Bytes = Bytes::from_slice(&e, "stellar".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let node_bytes: [u8; 32] = [
        47, 228, 204, 106, 21, 249, 70, 107, 173, 113, 237, 64, 122, 143, 27, 125, 168, 30, 253,
        147, 30, 119, 18, 117, 49, 82, 170, 23, 171, 192, 224, 110,
    ];

    let duration: u64 = test_data.min_duration * 2; // 2 years

    test_data.col_asset_stellar.mint(
        &new_owner,
        &(duration as i128 * test_data.node_rate as i128),
    );

    test_data.contract_client.set_record(
        &new_domain,
        &tld,
        &new_owner,
        &new_address_target,
        &duration,
    );

    let saved_record: Option<Record> = test_data
        .contract_client
        .record(&BytesN::from_array(&e, &node_bytes));

    assert!(saved_record.is_some());
    assert_eq!(
        saved_record.unwrap(),
        Record {
            node: BytesN::from_array(&e, &node_bytes),
            owner: new_owner.clone(),
            address: new_address_target.clone(),
            exp_date: e.ledger().timestamp() + duration,
            collateral: test_data.node_rate * (duration as u128),
        }
    );

    let error_already_created = test_data
        .contract_client
        .try_set_record(
            &new_domain,
            &tld,
            &new_owner,
            &new_address_target,
            &duration,
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(
        error_already_created,
        ContractErrors::RecordAlreadyExist.into()
    );
}

#[test]
fn test_unsupported_tld() {
    let e: Env = Env::default();
    e.mock_all_auths();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let new_owner: Address = Address::generate(&e);
    let new_domain: Bytes = Bytes::from_slice(&e, "stellar".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "eth".as_bytes());
    let duration: u64 = test_data.min_duration;

    let error = test_data
        .contract_client
        .try_set_record(&new_domain, &tld, &new_owner, &new_owner, &duration)
        .unwrap_err()
        .unwrap();

    assert_eq!(error, ContractErrors::UnsupportedTLD.into());
}

#[test]
fn test_invalid_duration() {
    let e: Env = Env::default();
    e.mock_all_auths();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let new_owner: Address = Address::generate(&e);
    let new_domain: Bytes = Bytes::from_slice(&e, "stellar".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = test_data.min_duration - 10;

    let error = test_data
        .contract_client
        .try_set_record(&new_domain, &tld, &new_owner, &new_owner, &duration)
        .unwrap_err()
        .unwrap();

    assert_eq!(error, ContractErrors::InvalidDuration.into());
}

#[test]
fn test_invalid_domain() {
    let e: Env = Env::default();
    e.mock_all_auths();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let new_owner: Address = Address::generate(&e);
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = test_data.min_duration - 10;

    let domain_with_numeric_value_error = test_data
        .contract_client
        .try_set_record(
            &Bytes::from_slice(&e, "stell4r".as_bytes()),
            &tld,
            &new_owner,
            &new_owner,
            &duration,
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(
        domain_with_numeric_value_error,
        ContractErrors::InvalidDomain.into()
    );

    let domain_with_uppercase_error = test_data
        .contract_client
        .try_set_record(
            &Bytes::from_slice(&e, "steIIar".as_bytes()),
            &tld,
            &new_owner,
            &new_owner,
            &duration,
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(
        domain_with_uppercase_error,
        ContractErrors::InvalidDomain.into()
    );

    let too_large_domain_error = test_data
        .contract_client
        .try_set_record(
            &Bytes::from_slice(
                &e,
                "thisisadomainsthatisnotvalidbecauseofhowlargeitis".as_bytes(),
            ),
            &tld,
            &new_owner,
            &new_owner,
            &duration,
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(too_large_domain_error, ContractErrors::InvalidDomain.into());
}

#[test]
fn test_multiplier() {
    let e: Env = Env::default();
    e.mock_all_auths();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);
}
