pub mod import {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32v1-none/release/registry_v2.wasm");
}

use crate::utils::GlobalTestData;
use import::{Client as RegistryV2ContractClient, WASM as RegistryV2Contract};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, Env};

pub struct RegistryV2TestData<'a> {
    pub registry_client: RegistryV2ContractClient<'a>,
    pub owner: Address,
    pub address: Address,
    pub domain: Bytes,
    pub periods: u64,
    pub tld: Bytes,
    pub node_bytes: [u8; 32],
}

impl RegistryV2TestData<'_> {
    pub fn create_test_data<'a>(e: &Env, global_test_data: &GlobalTestData) -> RegistryV2TestData<'a> {
        e.register_at(
            &global_test_data.registry_v2,
            RegistryV2Contract,
            (
                global_test_data.adm.clone(),
                global_test_data.oracle_addr.clone(),
                global_test_data.col_asset.clone(),
                soroban_sdk::vec![
                    &e,
                    Bytes::from(soroban_sdk::String::from_str(&e, "xlm")),
                    Bytes::from(soroban_sdk::String::from_str(&e, "lumens")),
                ],
                global_test_data.nfd.clone(),
                global_test_data.registry_v1.clone(),
            ),
        );

        RegistryV2TestData {
            registry_client: RegistryV2ContractClient::new(&e, &global_test_data.registry_v2),
            owner: Address::generate(&e),
            address: Address::generate(&e),
            domain: Bytes::from_slice(&e, "stellar".as_bytes()),
            periods: 2u64,
            tld: Bytes::from_slice(&e, "xlm".as_bytes()),
            node_bytes: [
                47, 228, 204, 106, 21, 249, 70, 107, 173, 113, 237, 64, 122, 143, 27, 125, 168, 30, 253, 147, 30, 119, 18, 117, 49, 82,
                170, 23, 171, 192, 224, 110,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::non_fungible_domain::NonFungibleDomainTestData;
    use crate::reflector;
    use crate::registry::RegistryV1TestData;
    use crate::registry_v2::RegistryV2TestData;
    use crate::registry_v2::import::{Domain, RecordKey, RegistryStorageKeys, RegistryV2Errors, SubDomain};
    use crate::utils::{GlobalTestData, create_global_test_data};
    use common::utils::generate_node;
    use soroban_sdk::testutils::{Address as _, Events, Ledger, MockAuth, MockAuthInvoke};
    use soroban_sdk::{Address, Bytes, BytesN, Env, FromVal, IntoVal, Map, String, Symbol, Val, symbol_short, vec};

    #[test]
    fn test_registering_a_domain() {
        let e: Env = Env::default();
        e.ledger().set_timestamp(900);

        let global_test_data: GlobalTestData = create_global_test_data(&e);

        let (oracle_client, _) = reflector::init_reflector(&e, &global_test_data.adm, &global_test_data.oracle_addr);
        oracle_client
            .mock_all_auths()
            .set_price(&vec![&e, 2376i128 * 10i128.pow(10)], &600_000);

        let registry_test_data: RegistryV2TestData = RegistryV2TestData::create_test_data(&e, &global_test_data);
        let nfd_test_data: NonFungibleDomainTestData = NonFungibleDomainTestData::create_test_data(&e, &global_test_data);

        let invalid_domain_error = registry_test_data
            .registry_client
            .mock_all_auths()
            .try_register(
                &Bytes::from_slice(&e, "stell4r".as_bytes()),
                &registry_test_data.tld,
                &registry_test_data.owner,
                &registry_test_data.address,
                &2,
            )
            .unwrap_err()
            .unwrap();

        assert_eq!(RegistryV2Errors::InvalidDomain, invalid_domain_error);

        let unsupported_tld = registry_test_data
            .registry_client
            .mock_all_auths()
            .try_register(
                &registry_test_data.domain,
                &Bytes::from_slice(&e, "lll".as_bytes()),
                &registry_test_data.owner,
                &registry_test_data.address,
                &2,
            )
            .unwrap_err()
            .unwrap();

        assert_eq!(RegistryV2Errors::UnsupportedTLD, unsupported_tld);

        let payment_failed_error = registry_test_data
            .registry_client
            .mock_all_auths()
            .try_register(
                &registry_test_data.domain,
                &registry_test_data.tld,
                &registry_test_data.owner,
                &registry_test_data.address,
                &registry_test_data.periods,
            )
            .unwrap_err()
            .unwrap();

        assert_eq!(RegistryV2Errors::PaymentFailed, payment_failed_error);

        global_test_data
            .col_asset_stellar
            .mock_all_auths()
            .mint(&registry_test_data.owner, &100_0000000i128);

        assert!(
            registry_test_data
                .registry_client
                .try_register(
                    &registry_test_data.domain,
                    &registry_test_data.tld,
                    &registry_test_data.owner,
                    &registry_test_data.address,
                    &registry_test_data.periods
                )
                .is_err()
        );

        registry_test_data
            .registry_client
            .mock_auths(&[MockAuth {
                address: &registry_test_data.owner,
                invoke: &MockAuthInvoke {
                    contract: &registry_test_data.registry_client.address,
                    fn_name: "register",
                    args: (
                        registry_test_data.domain.clone(),
                        registry_test_data.tld.clone(),
                        registry_test_data.owner.clone(),
                        registry_test_data.address.clone(),
                        2u64,
                    )
                        .into_val(&e),
                    sub_invokes: &[MockAuthInvoke {
                        contract: &global_test_data.col_asset,
                        fn_name: "transfer",
                        args: (
                            registry_test_data.owner.clone(),
                            registry_test_data.registry_client.address.clone(),
                            841750841i128,
                        )
                            .into_val(&e),
                        sub_invokes: &[],
                    }],
                },
            }])
            .register(
                &registry_test_data.domain,
                &registry_test_data.tld,
                &registry_test_data.owner,
                &registry_test_data.address,
                &registry_test_data.periods,
            );

        let event = e.events().all().last().unwrap();
        assert_eq!(event.0, registry_test_data.registry_client.address);
        assert_eq!(event.1, vec![&e, symbol_short!("REGISTRY"), symbol_short!("DOMAIN")].into_val(&e));

        let event_map: Map<Symbol, Val> = Map::from_val(&e, &event.2);
        assert_eq!(
            Address::from_val(&e, &event_map.get(symbol_short!("register")).unwrap()),
            registry_test_data.owner
        );
        assert_eq!(
            Bytes::from_val(&e, &event_map.get(symbol_short!("domain")).unwrap()),
            registry_test_data.domain
        );
        assert_eq!(
            Bytes::from_val(&e, &event_map.get(symbol_short!("tld")).unwrap()),
            registry_test_data.tld
        );
        assert_eq!(
            Address::from_val(&e, &event_map.get(symbol_short!("address")).unwrap()),
            registry_test_data.address
        );
        assert_eq!(
            u64::from_val(&e, &event_map.get(symbol_short!("exp_date")).unwrap()),
            900 + (3600 * 24 * 360 * 2)
        );
        assert_eq!(
            u128::from_val(&e, &event_map.get(Symbol::new(&e, "amount_paid")).unwrap()),
            841750841u128
        );

        e.as_contract(&registry_test_data.registry_client.address, || {
            let record: Domain = e
                .storage()
                .persistent()
                .get(&RegistryStorageKeys::Domain(BytesN::from_array(&e, &registry_test_data.node_bytes)))
                .unwrap();

            assert_eq!(
                record,
                Domain {
                    domain: registry_test_data.domain.clone(),
                    tld: registry_test_data.tld.clone(),
                    node: BytesN::from_array(&e, &registry_test_data.node_bytes).clone(),
                    address: registry_test_data.address.clone(),
                    token_id: 1.clone(),
                    snapshot: e.ledger().timestamp().clone(),
                    exp_date: 900 + (3600 * 24 * 360 * 2),
                }
            );
        });

        let already_exists_error = registry_test_data
            .registry_client
            .mock_all_auths()
            .try_register(
                &registry_test_data.domain,
                &registry_test_data.tld,
                &registry_test_data.owner,
                &registry_test_data.address,
                &registry_test_data.periods,
            )
            .unwrap_err()
            .unwrap();

        assert_eq!(RegistryV2Errors::DomainAlreadyExist, already_exists_error);

        e.as_contract(&registry_test_data.registry_client.address, || {
            let index: u32 = e.storage().instance().get(&RegistryStorageKeys::Index).unwrap();
            assert_eq!(index, 1u32);
        });

        // We check that values from the nfd contract are correct
        assert_eq!(nfd_test_data.nfd_client.balance(&registry_test_data.owner), 1);
        assert_eq!(nfd_test_data.nfd_client.owner_of(&1), registry_test_data.owner);
    }

    #[test]
    pub fn test_v1_existing_domain() {
        let e: Env = Env::default();
        e.ledger().set_timestamp(900);

        let global_test_data: GlobalTestData = create_global_test_data(&e);
        let v2_registry_test_data: RegistryV2TestData = RegistryV2TestData::create_test_data(&e, &global_test_data);
        NonFungibleDomainTestData::create_test_data(&e, &global_test_data);

        let v1_test_data: RegistryV1TestData = RegistryV1TestData::create_test_data(&e, &global_test_data);
        RegistryV1TestData::init_contract(&global_test_data, &v1_test_data);

        let (oracle_client, _) = reflector::init_reflector(&e, &global_test_data.adm, &global_test_data.oracle_addr);
        oracle_client
            .mock_all_auths()
            .set_price(&vec![&e, 2376i128 * 10i128.pow(10)], &600_000);

        let duration: u64 = v1_test_data.min_duration * 2; // 2 years

        global_test_data
            .col_asset_stellar
            .mock_all_auths()
            .mint(&v2_registry_test_data.owner, &(841750841i128 * 10));

        v1_test_data.contract_client.mock_all_auths().set_record(
            &v2_registry_test_data.domain,
            &v2_registry_test_data.tld,
            &v2_registry_test_data.owner,
            &v2_registry_test_data.address,
            &duration,
        );

        let periods: u64 = 2;

        let v1_registered_error = v2_registry_test_data
            .registry_client
            .mock_all_auths()
            .try_register(
                &v2_registry_test_data.domain,
                &v2_registry_test_data.tld,
                &v2_registry_test_data.owner,
                &v2_registry_test_data.address,
                &periods,
            )
            .unwrap_err()
            .unwrap();

        assert_eq!(v1_registered_error, RegistryV2Errors::V1DomainRegistered);
    }

    #[test]
    pub fn test_registering_a_subdomain() {
        let e: Env = Env::default();
        e.ledger().set_timestamp(900);

        let global_test_data: GlobalTestData = create_global_test_data(&e);
        let registry_test_data: RegistryV2TestData = RegistryV2TestData::create_test_data(&e, &global_test_data);
        let nfd_test_data: NonFungibleDomainTestData = NonFungibleDomainTestData::create_test_data(&e, &global_test_data);

        let (oracle_client, _) = reflector::init_reflector(&e, &global_test_data.adm, &global_test_data.oracle_addr);
        oracle_client
            .mock_all_auths()
            .set_price(&vec![&e, 2376i128 * 10i128.pow(10)], &600_000);

        global_test_data
            .col_asset_stellar
            .mock_all_auths()
            .mint(&registry_test_data.owner, &841750841i128);

        registry_test_data.registry_client.mock_all_auths().register(
            &registry_test_data.domain,
            &registry_test_data.tld,
            &registry_test_data.owner,
            &registry_test_data.address,
            &registry_test_data.periods,
        );

        let invalid_subdomain_error = registry_test_data
            .registry_client
            .mock_all_auths()
            .try_register_sub(
                &RecordKey::Domain(BytesN::from_array(&e, &registry_test_data.node_bytes)),
                &Bytes::from_slice(&e, "dao12".as_bytes()),
                &registry_test_data.address,
            )
            .unwrap_err()
            .unwrap();

        assert_eq!(RegistryV2Errors::InvalidDomain, invalid_subdomain_error);

        let non_existing_domain_parent = registry_test_data
            .registry_client
            .mock_all_auths()
            .try_register_sub(
                &RecordKey::Domain(generate_node(
                    &e,
                    &Bytes::from_slice(&e, "dao".as_bytes()),
                    &Bytes::from_slice(&e, "daol".as_bytes()),
                )),
                &Bytes::from_slice(&e, "dao".as_bytes()),
                &registry_test_data.address,
            )
            .unwrap_err()
            .unwrap();

        assert_eq!(RegistryV2Errors::RecordDoesntExist, non_existing_domain_parent);

        let non_existing_subdomain_parent = registry_test_data
            .registry_client
            .mock_all_auths()
            .try_register_sub(
                &RecordKey::SubDomain(generate_node(
                    &e,
                    &Bytes::from_slice(&e, "dao".as_bytes()),
                    &Bytes::from_slice(&e, "daol".as_bytes()),
                )),
                &Bytes::from_slice(&e, "dao".as_bytes()),
                &registry_test_data.address,
            )
            .unwrap_err()
            .unwrap();

        assert_eq!(RegistryV2Errors::RecordDoesntExist, non_existing_subdomain_parent);

        let new_subdomain_address: Address = Address::generate(&e);
        let new_subdomain: Bytes = Bytes::from_slice(&e, "dao".as_bytes());
        let new_subdomain_node: BytesN<32> = generate_node(&e, &new_subdomain, &Bytes::from_array(&e, &registry_test_data.node_bytes));

        registry_test_data
            .registry_client
            .mock_auths(&[MockAuth {
                address: &registry_test_data.owner,
                invoke: &MockAuthInvoke {
                    contract: &registry_test_data.registry_client.address,
                    fn_name: "register_sub",
                    args: (
                        RecordKey::Domain(BytesN::from_array(&e, &registry_test_data.node_bytes)),
                        new_subdomain.clone(),
                        new_subdomain_address.clone(),
                    )
                        .into_val(&e),
                    sub_invokes: &[],
                },
            }])
            .register_sub(
                &RecordKey::Domain(BytesN::from_array(&e, &registry_test_data.node_bytes)),
                &new_subdomain,
                &new_subdomain_address,
            );

        let event = e.events().all().last().unwrap();
        assert_eq!(event.0, registry_test_data.registry_client.address);
        assert_eq!(
            event.1,
            vec![&e, symbol_short!("REGISTRY"), symbol_short!("SUBDOMAIN")].into_val(&e)
        );

        let event_map: Map<Symbol, Val> = Map::from_val(&e, &event.2);
        assert_eq!(Bytes::from_val(&e, &event_map.get(symbol_short!("domain")).unwrap()), new_subdomain);
        assert_eq!(
            BytesN::from_val(&e, &event_map.get(symbol_short!("parent")).unwrap()),
            BytesN::from_array(&e, &registry_test_data.node_bytes)
        );
        assert_eq!(
            Address::from_val(&e, &event_map.get(symbol_short!("address")).unwrap()),
            new_subdomain_address
        );

        let subdomain_already_exist_error = registry_test_data
            .registry_client
            .mock_all_auths()
            .try_register_sub(
                &RecordKey::Domain(BytesN::from_array(&e, &registry_test_data.node_bytes)),
                &new_subdomain,
                &new_subdomain_address,
            )
            .unwrap_err()
            .unwrap();

        assert_eq!(RegistryV2Errors::DomainAlreadyExist, subdomain_already_exist_error);

        e.as_contract(&registry_test_data.registry_client.address, || {
            let parent: Domain = e
                .storage()
                .persistent()
                .get(&RegistryStorageKeys::Domain(BytesN::from_array(&e, &registry_test_data.node_bytes)))
                .unwrap();
            let record: SubDomain = e
                .storage()
                .persistent()
                .get(&RegistryStorageKeys::SubDomain(new_subdomain_node.clone()))
                .unwrap();
            assert_eq!(
                record,
                SubDomain {
                    parent: BytesN::from_array(&e, &registry_test_data.node_bytes).clone(),
                    domain: new_subdomain.clone(),
                    node: new_subdomain_node.clone(),
                    address: new_subdomain_address.clone(),
                    root: parent.node.clone(),
                    snapshot: parent.snapshot.clone(),
                }
            );
        });

        // If the owner transfers the domain to another account, it shouldn't be able to set a new subdomain
        let new_owner: Address = Address::generate(&e);

        nfd_test_data
            .nfd_client
            .mock_all_auths()
            .transfer(&registry_test_data.owner, &new_owner, &1);

        assert!(
            registry_test_data
                .registry_client
                .mock_auths(&[MockAuth {
                    address: &registry_test_data.owner,
                    invoke: &MockAuthInvoke {
                        contract: &registry_test_data.registry_client.address,
                        fn_name: "register_sub",
                        args: (
                            RecordKey::Domain(BytesN::from_array(&e, &registry_test_data.node_bytes)),
                            String::from_str(&e, "hello").to_bytes(),
                            new_subdomain_address.clone(),
                        )
                            .into_val(&e),
                        sub_invokes: &[],
                    },
                }])
                .try_register_sub(
                    &RecordKey::Domain(BytesN::from_array(&e, &registry_test_data.node_bytes)),
                    &String::from_str(&e, "hello").to_bytes(),
                    &new_subdomain_address,
                )
                .is_err()
        );
    }
}
