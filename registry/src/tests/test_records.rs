#![cfg(test)]

use crate::errors::ContractErrors;
use crate::storage::record::{Domain, Record, RecordKeys, SubDomain};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Bytes, BytesN, Env, IntoVal};
use crate::utils::records::generate_node;

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
        .record(&RecordKeys::Record(BytesN::from_array(&e, &node_bytes)));

    assert!(saved_record.is_some());
    assert_eq!(
        saved_record.unwrap(),
        Record::Domain(Domain {
            node: BytesN::from_array(&e, &node_bytes),
            owner: new_owner.clone(),
            address: new_address_target.clone(),
            exp_date: e.ledger().timestamp() + duration,
            collateral: test_data.node_rate * (duration as u128),
            snapshot: e.ledger().timestamp(),
        })
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
    // TODO:
}

#[test]
fn test_subdomains() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let owner: Address = Address::generate(&e);
    let domain_address: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "stellar".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = test_data.min_duration;

    test_data.col_asset_stellar.mock_all_auths().mint(
        &owner,
        &(duration as i128 * test_data.node_rate as i128),
    );

    test_data.contract_client
        .mock_all_auths()
        .set_record(
            &domain,
            &tld,
            &owner,
            &domain_address,
            &duration,
        );

    let sub_domain: Bytes = Bytes::from_slice(&e, "payments".as_bytes());
    let domain_node: BytesN<32> = generate_node(&e, &domain, &tld);
    let new_address: Address = Address::generate(&e);

    test_data.contract_client
        .mock_auths(&[
            MockAuth {
                address: &owner,
                invoke: &MockAuthInvoke {
                    contract: &test_data.contract_client.address,
                    fn_name: "set_sub",
                    args: (
                        sub_domain.clone(),
                        RecordKeys::Record(domain_node.clone()),
                        new_address.clone()
                    ).into_val(&e),
                    sub_invokes: &[],
                },
            }
        ])
        .set_sub(&sub_domain, &RecordKeys::Record(domain_node.clone()), &new_address);

    let sub_domain_node: BytesN<32> = generate_node(&e, &sub_domain, &Bytes::from(domain_node.clone()));

    let sub_domain_record: SubDomain = match test_data.contract_client.record(&RecordKeys::SubRecord(sub_domain_node.clone())).unwrap() {
        Record::Domain(_) => panic!(),
        Record::SubDomain(sub) => sub,
    };


    assert_eq!(sub_domain_record.address, new_address);
    assert_eq!(sub_domain_record.parent, domain_node);

    // If there is no signature from the owner of the root domain, it can not update the subdomain
    assert!(
        test_data.contract_client
            .mock_auths(&[
                MockAuth {
                    address: &Address::generate(&e),
                    invoke: &MockAuthInvoke {
                        contract: &test_data.contract_client.address,
                        fn_name: "set_sub",
                        args: (
                            sub_domain.clone(),
                            RecordKeys::Record(domain_node.clone()),
                            new_address.clone()
                        ).into_val(&e),
                        sub_invokes: &[],
                    },
                }
            ])
            .try_set_sub(&sub_domain, &RecordKeys::Record(domain_node.clone()), &new_address)
            .is_err()
    );

    let sub_domain: Bytes = Bytes::from_slice(&e, "payments".as_bytes());
    let domain_node: BytesN<32> = generate_node(&e, &domain, &tld);
    let updated_address: Address = Address::generate(&e);

    test_data.contract_client
        .mock_all_auths()
        .set_sub(&sub_domain, &RecordKeys::Record(domain_node.clone()), &updated_address);

    let updated_sub_domain_record: SubDomain = match test_data.contract_client.record(&RecordKeys::SubRecord(sub_domain_node.clone())).unwrap() {
        Record::Domain(_) => panic!(),
        Record::SubDomain(sub) => sub,
    };

    assert_eq!(updated_sub_domain_record.address, updated_address);
}

#[test]
fn test_updating_address() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let owner: Address = Address::generate(&e);
    let address: Address = Address::generate(&e);
    let second_address: Address = Address::generate(&e);

    let new_domain: Bytes = Bytes::from_slice(&e, "stellar".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());

    test_data.col_asset_stellar.mock_all_auths().mint(
        &owner,
        &(test_data.min_duration as i128 * test_data.node_rate as i128),
    );

    test_data.contract_client.mock_all_auths().set_record(
        &new_domain,
        &tld,
        &owner,
        &address,
        &test_data.min_duration,
    );

    let node: BytesN<32> = test_data.contract_client.parse_domain(&new_domain, &tld);

    let first_record: Domain = match test_data.contract_client.record(&RecordKeys::Record(node.clone())).unwrap() {
        Record::Domain(domain) => domain,
        Record::SubDomain(_) => panic!()
    };

    test_data.contract_client.mock_auths(&[MockAuth {
        address: &owner,
        invoke: &MockAuthInvoke {
            contract: &test_data.contract_client.address,
            fn_name: "update_address",
            args: (RecordKeys::Record(node.clone()), second_address.clone()).into_val(&e),
            sub_invokes: &[],
        },
    }]).update_address(&RecordKeys::Record(node.clone()), &second_address);

    let second_record: Domain = match test_data.contract_client.record(&RecordKeys::Record(node.clone())).unwrap() {
        Record::Domain(domain) => domain,
        Record::SubDomain(_) => panic!()
    };

    assert_eq!(address, first_record.address);
    assert_eq!(second_address, second_record.address);
}
