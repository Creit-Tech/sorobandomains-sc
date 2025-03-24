#![cfg(test)]

use crate::storage::record::{Domain, Record, RecordKeys};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Bytes, BytesN, Env, IntoVal};
use test_utils::create_env;

#[test]
fn test_simple_transfer() {
    let e: Env = create_env();
    let test_data: TestData = create_test_data(&e);
    init_contract(&e, &test_data);

    let first_owner: Address = Address::generate(&e);
    let second_owner: Address = Address::generate(&e);

    let new_domain: Bytes = Bytes::from_slice(&e, "stellar".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());

    test_data.col_asset_stellar.mock_all_auths().mint(
        &first_owner,
        &(test_data.min_duration as i128 * test_data.node_rate as i128),
    );

    test_data.contract_client.mock_all_auths().set_record(
        &new_domain,
        &tld,
        &first_owner,
        &first_owner,
        &test_data.min_duration,
    );

    let node: BytesN<32> = test_data.contract_client.parse_domain(&new_domain, &tld);

    let first_record: Domain = match test_data
        .contract_client
        .record(&RecordKeys::Record(node.clone()))
        .unwrap()
    {
        Record::Domain(domain) => domain,
        Record::SubDomain(_) => panic!(),
    };

    e.ledger().set_timestamp(1742825701 + 10);

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &first_owner,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "transfer",
                args: (RecordKeys::Record(node.clone()), second_owner.clone()).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .transfer(&RecordKeys::Record(node.clone()), &second_owner);

    let second_record: Domain = match test_data
        .contract_client
        .record(&RecordKeys::Record(node.clone()))
        .unwrap()
    {
        Record::Domain(domain) => domain,
        Record::SubDomain(_) => panic!(),
    };

    assert_eq!(first_owner, first_record.owner);
    assert_eq!(1742825701, first_record.snapshot);
    assert_eq!(second_owner, second_record.owner);
    assert_eq!(1742825701 + 10, second_record.snapshot);

    // It should fail because first owner is no the owner anymore
    assert!(test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &first_owner,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "transfer",
                args: (RecordKeys::Record(node.clone()), second_owner.clone()).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .try_transfer(&RecordKeys::Record(node.clone()), &second_owner)
        .is_err());
}
