#![cfg(test)]

use common::utils::generate_node;
use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Bytes, BytesN, Env, IntoVal, Vec,
};
use test_utils::{
    create_env, create_global_test_data,
    registry::{self},
    registry_contract::RecordKeys,
    reverse_registrar,
    reverse_registrar_contract::{Domain, Error},
    GlobalTestData,
};

use crate::events::EventTopics;

#[test]
fn test_set_new_domain_with_domain() {
    let e: Env = create_env();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain_address: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "reversedemo".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = registry_test_data.min_duration;

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    global_test_data
        .gov_asset_stellar
        .mock_all_auths()
        .mint(&domain_address, &i128::MAX);

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain, &tld, &owner, &domain_address, &duration);

    let domain = Domain {
        tld,
        sld: domain,
        subs: Vec::new(&e),
    };
    reverse_registrar_test_data
        .contract_client
        .mock_all_auths()
        .set(&domain_address, &Some(domain.clone()));

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .get(&domain_address),
        Some(domain.clone())
    );
    reverse_registrar_test_data
        .contract_client
        .mock_all_auths()
        .set(&domain_address, &None);

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .get(&domain_address),
        None
    );

    let mut filtered_events = soroban_sdk::Vec::new(&e);
    for (contract_id, topics, data) in e.events().all().iter() {
        if contract_id == reverse_registrar_test_data.contract_client.address {
            filtered_events.push_back((topics.clone(), data.clone()));
        }
    }
    assert_eq!(
        filtered_events,
        vec![
            &e,
            (
                (EventTopics::DomainUpdated,).into_val(&e),
                (domain_address.clone(), Some(domain.clone()),).into_val(&e)
            ),
            (
                (EventTopics::DomainUpdated,).into_val(&e),
                (domain_address.clone(), None::<Domain>,).into_val(&e)
            ),
        ]
    );

    assert_eq!(
        global_test_data
            .gov_asset_client
            .mock_all_auths()
            .balance(&domain_address),
        i128::MAX - reverse_registrar_test_data.fee
    )
}

#[test]
fn test_set_new_domain_with_subdomain() {
    let e: Env = create_env();
    e.budget().reset_unlimited();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "reversedemo".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = registry_test_data.min_duration;

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain, &tld, &owner, &owner, &duration);

    let sub_domain: Bytes = Bytes::from_slice(&e, "payments".as_bytes());
    let domain_node: BytesN<32> = generate_node(&e, &domain, &tld);
    let domain_address: Address = Address::generate(&e);

    global_test_data
        .gov_asset_stellar
        .mock_all_auths()
        .mint(&domain_address, &i128::MAX);

    registry_test_data.contract_client.mock_all_auths().set_sub(
        &sub_domain,
        &RecordKeys::Record(domain_node.clone()),
        &domain_address,
    );

    let domain = Domain {
        tld,
        sld: domain,
        subs: vec![&e, Bytes::from_slice(&e, "payments".as_bytes())],
    };

    reverse_registrar_test_data
        .contract_client
        .mock_all_auths()
        .set(&domain_address, &Some(domain.clone()));

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .get(&domain_address),
        Some(domain.clone())
    );

    reverse_registrar_test_data
        .contract_client
        .mock_all_auths()
        .set(&domain_address, &None);

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .get(&domain_address),
        None
    );
    let mut filtered_events = soroban_sdk::Vec::new(&e);
    for (contract_id, topics, data) in e.events().all().iter() {
        if contract_id == reverse_registrar_test_data.contract_client.address {
            filtered_events.push_back((topics.clone(), data.clone()));
        }
    }
    assert_eq!(
        filtered_events,
        vec![
            &e,
            (
                (EventTopics::DomainUpdated,).into_val(&e),
                (domain_address.clone(), Some(domain.clone()),).into_val(&e)
            ),
            (
                (EventTopics::DomainUpdated,).into_val(&e),
                (domain_address.clone(), None::<Domain>,).into_val(&e)
            ),
        ]
    );

    assert_eq!(
        global_test_data
            .gov_asset_client
            .mock_all_auths()
            .balance(&domain_address),
        i128::MAX - reverse_registrar_test_data.fee
    )
}

#[test]
fn test_set_same_domain_should_only_bump() {
    let e: Env = create_env();
    e.budget().reset_unlimited();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain_address: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "reversedemo".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = registry_test_data.min_duration;

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    global_test_data
        .gov_asset_stellar
        .mock_all_auths()
        .mint(&domain_address, &i128::MAX);

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain, &tld, &owner, &domain_address, &duration);

    let domain = Domain {
        tld,
        sld: domain,
        subs: Vec::new(&e),
    };

    reverse_registrar_test_data
        .contract_client
        .mock_all_auths()
        .set(&domain_address, &Some(domain.clone()));

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .get(&domain_address),
        Some(domain.clone())
    );

    reverse_registrar_test_data
        .contract_client
        .mock_all_auths()
        .set(&domain_address, &Some(domain.clone()));

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .get(&domain_address),
        Some(domain.clone())
    );

    let mut filtered_events = soroban_sdk::Vec::new(&e);
    for (contract_id, topics, data) in e.events().all().iter() {
        if contract_id == reverse_registrar_test_data.contract_client.address {
            filtered_events.push_back((topics.clone(), data.clone()));
        }
    }
    assert_eq!(
        filtered_events,
        vec![
            &e,
            (
                (EventTopics::DomainUpdated,).into_val(&e),
                (domain_address.clone(), Some(domain.clone()),).into_val(&e)
            ),
        ]
    );

    assert_eq!(
        global_test_data
            .gov_asset_client
            .mock_all_auths()
            .balance(&domain_address),
        i128::MAX - reverse_registrar_test_data.fee
    )
}

#[test]
fn test_set_new_domain_should_update() {
    let e: Env = create_env();
    e.budget().reset_unlimited();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain_address: Address = Address::generate(&e);
    let domain1: Bytes = Bytes::from_slice(&e, "reversedemoa".as_bytes());
    let domain2: Bytes = Bytes::from_slice(&e, "reversedemob".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = registry_test_data.min_duration;

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    global_test_data
        .gov_asset_stellar
        .mock_all_auths()
        .mint(&domain_address, &i128::MAX);

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain1, &tld, &owner, &domain_address, &duration);

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain2, &tld, &owner, &domain_address, &duration);

    let domain1 = Domain {
        tld: tld.clone(),
        sld: domain1,
        subs: Vec::new(&e),
    };

    reverse_registrar_test_data
        .contract_client
        .mock_all_auths()
        .set(&domain_address, &Some(domain1.clone()));

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .get(&domain_address),
        Some(domain1.clone())
    );

    let domain2 = Domain {
        tld,
        sld: domain2,
        subs: Vec::new(&e),
    };

    reverse_registrar_test_data
        .contract_client
        .mock_all_auths()
        .set(&domain_address, &Some(domain2.clone()));

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .get(&domain_address),
        Some(domain2.clone())
    );

    let mut filtered_events = soroban_sdk::Vec::new(&e);
    for (contract_id, topics, data) in e.events().all().iter() {
        if contract_id == reverse_registrar_test_data.contract_client.address {
            filtered_events.push_back((topics.clone(), data.clone()));
        }
    }
    assert_eq!(
        filtered_events,
        vec![
            &e,
            (
                (EventTopics::DomainUpdated,).into_val(&e),
                (domain_address.clone(), Some(domain1.clone()),).into_val(&e)
            ),
            (
                (EventTopics::DomainUpdated,).into_val(&e),
                (domain_address.clone(), Some(domain2.clone()),).into_val(&e)
            ),
        ]
    );

    assert_eq!(
        global_test_data
            .gov_asset_client
            .mock_all_auths()
            .balance(&domain_address),
        i128::MAX - reverse_registrar_test_data.fee * 2
    )
}

#[test]
fn test_remove_nonexistent_domain_should_do_nothing() {
    let e: Env = create_env();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain_address: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "reversedemo".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = registry_test_data.min_duration;

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    global_test_data
        .gov_asset_stellar
        .mock_all_auths()
        .mint(&domain_address, &i128::MAX);

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain, &tld, &owner, &domain_address, &duration);

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .get(&domain_address),
        None
    );
    reverse_registrar_test_data
        .contract_client
        .mock_all_auths()
        .set(&domain_address, &None);

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .get(&domain_address),
        None
    );

    let mut filtered_events = soroban_sdk::Vec::new(&e);
    for (contract_id, topics, data) in e.events().all().iter() {
        if contract_id == reverse_registrar_test_data.contract_client.address {
            filtered_events.push_back((topics.clone(), data.clone()));
        }
    }
    assert!(filtered_events.is_empty());

    assert_eq!(
        global_test_data
            .gov_asset_client
            .mock_all_auths()
            .balance(&domain_address),
        i128::MAX
    )
}

#[test]
fn test_address_missmatch_error_with_domain() {
    let e: Env = create_env();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain_address: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "reversedemo".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = registry_test_data.min_duration;

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain, &tld, &owner, &domain_address, &duration);

    let domain = Domain {
        tld,
        sld: domain,
        subs: Vec::new(&e),
    };

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .mock_all_auths()
            .try_set(&owner, &Some(domain.clone()))
            .unwrap_err()
            .unwrap(),
        Error::AddressMismatch.into()
    )
}

#[test]
fn test_address_missmatch_error_with_subdomain() {
    let e: Env = create_env();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "reversedemo".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = registry_test_data.min_duration;

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain, &tld, &owner, &owner, &duration);

    let sub_domain: Bytes = Bytes::from_slice(&e, "payments".as_bytes());
    let domain_node: BytesN<32> = generate_node(&e, &domain, &tld);
    let domain_address: Address = Address::generate(&e);

    registry_test_data.contract_client.mock_all_auths().set_sub(
        &sub_domain,
        &RecordKeys::Record(domain_node.clone()),
        &domain_address,
    );

    let domain = Domain {
        tld,
        sld: domain.clone(),
        subs: vec![&e, Bytes::from_slice(&e, "payments".as_bytes())],
    };

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .mock_all_auths()
            .try_set(&owner, &Some(domain.clone()))
            .unwrap_err()
            .unwrap(),
        Error::AddressMismatch.into()
    )
}

#[test]
fn test_not_implemented_error() {
    let e: Env = create_env();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "reversedemo".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = registry_test_data.min_duration;

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain, &tld, &owner, &owner, &duration);

    let sub_domain: Bytes = Bytes::from_slice(&e, "payments".as_bytes());
    let domain_node: BytesN<32> = generate_node(&e, &domain, &tld);
    let domain_address: Address = Address::generate(&e);

    registry_test_data.contract_client.mock_all_auths().set_sub(
        &sub_domain,
        &RecordKeys::Record(domain_node.clone()),
        &domain_address,
    );

    let domain = Domain {
        tld,
        sld: domain.clone(),
        subs: vec![
            &e,
            Bytes::from_slice(&e, "payments".as_bytes()),
            Bytes::from_slice(&e, "hello".as_bytes()),
        ],
    };

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .mock_all_auths()
            .try_set(&owner, &Some(domain.clone()))
            .unwrap_err()
            .unwrap(),
        Error::NotImplemented.into()
    )
}

#[test]
fn test_failed_to_get_record_with_domain() {
    let e: Env = create_env();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "reversedemo".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    let domain = Domain {
        tld,
        sld: domain,
        subs: Vec::new(&e),
    };

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .mock_all_auths()
            .try_set(&owner, &Some(domain.clone()))
            .unwrap_err()
            .unwrap(),
        Error::FailedToGetRecord.into()
    )
}

#[test]
fn test_failed_to_get_record_error_with_subdomain() {
    let e: Env = create_env();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "reversedemo".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = registry_test_data.min_duration;

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain, &tld, &owner, &owner, &duration);

    let domain = Domain {
        tld,
        sld: domain,
        subs: vec![&e, Bytes::from_slice(&e, "payments".as_bytes())],
    };

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .mock_all_auths()
            .try_set(&owner, &Some(domain.clone()))
            .unwrap_err()
            .unwrap(),
        Error::FailedToGetRecord.into()
    )
}

#[test]
fn test_failed_to_pay_fee_error() {
    let e: Env = create_env();
    let global_test_data: GlobalTestData = create_global_test_data(&e);
    let registry_test_data: registry::TestData = registry::create_test_data(&e);
    let reverse_registrar_test_data = reverse_registrar::create_test_data(&e);
    registry::init_contract(&global_test_data, &registry_test_data);
    reverse_registrar::init_contract(
        &global_test_data,
        &registry_test_data,
        &reverse_registrar_test_data,
    );

    let owner: Address = Address::generate(&e);
    let domain_address: Address = Address::generate(&e);
    let domain: Bytes = Bytes::from_slice(&e, "reversedemo".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());
    let duration: u64 = registry_test_data.min_duration;

    global_test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&owner, &i128::MAX);

    global_test_data
        .gov_asset_stellar
        .mock_all_auths()
        .mint(&domain_address, &(reverse_registrar_test_data.fee - 1));

    registry_test_data
        .contract_client
        .mock_all_auths()
        .set_record(&domain, &tld, &owner, &domain_address, &duration);

    let domain = Domain {
        tld,
        sld: domain,
        subs: Vec::new(&e),
    };

    assert_eq!(
        reverse_registrar_test_data
            .contract_client
            .mock_all_auths()
            .try_set(&domain_address, &Some(domain.clone()))
            .unwrap_err()
            .unwrap(),
        Error::FailedToPayFee.into()
    );

    assert_eq!(
        global_test_data
            .gov_asset_client
            .mock_all_auths()
            .balance(&domain_address),
        reverse_registrar_test_data.fee - 1
    )
}
