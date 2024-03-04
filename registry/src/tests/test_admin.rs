#![cfg(test)]

use crate::storage::core::{CoreData, CoreDataEntity};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::{MockAuth, MockAuthInvoke};
use soroban_sdk::{Bytes, Env, IntoVal, Vec};

#[test]
pub fn test_updating_tlds() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let core: CoreData = e.as_contract(&test_data.contract_client.address, || {
        e.core_data().unwrap()
    });

    assert_eq!(core.allowed_tlds, test_data.allowed_tlds);

    // It should fail because the admin didn't sign the transaction
    assert!(test_data
        .contract_client
        .try_update_tlds(&Vec::from_array(
            &e,
            [Bytes::from_slice(&e, "eth".as_bytes())]
        ))
        .is_err());

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &test_data.adm,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "update_tlds",
                args: (Vec::from_array(
                    &e,
                    [Bytes::from_slice(&e, "eth".as_bytes())],
                ),)
                    .into_val(&e),
                sub_invokes: &[],
            },
        }])
        .update_tlds(&Vec::from_array(
            &e,
            [Bytes::from_slice(&e, "eth".as_bytes())],
        ));

    let updated_core: CoreData = e.as_contract(&test_data.contract_client.address, || {
        e.core_data().unwrap()
    });

    assert_eq!(
        updated_core.allowed_tlds,
        Vec::from_array(&e, [Bytes::from_slice(&e, "eth".as_bytes())])
    );
}
