use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

pub mod import {
    use soroban_sdk::contractimport;
    contractimport!(file = "../target/wasm32v1-none/release/non_fungible_domain.wasm");
}

use crate::utils::GlobalTestData;
pub use import::{Client as NonFungibleDomainClient, WASM as NonFungibleDomain};

pub struct NonFungibleDomainTestData<'a> {
    pub owner_1: Address,
    pub owner_2: Address,
    pub nfd_client: NonFungibleDomainClient<'a>,
}

impl NonFungibleDomainTestData<'_> {
    pub fn create_test_data<'a>(e: &Env, global_test_data: &GlobalTestData) -> NonFungibleDomainTestData<'a> {
        e.register_at(
            &global_test_data.nfd,
            NonFungibleDomain,
            (global_test_data.adm.clone(), global_test_data.registry_v2.clone()),
        );

        NonFungibleDomainTestData {
            owner_1: Address::generate(&e),
            owner_2: Address::generate(&e),
            nfd_client: NonFungibleDomainClient::new(&e, &global_test_data.nfd),
        }
    }
}

#[cfg(test)]
pub mod tests_utils {
    use crate::non_fungible_domain::NonFungibleDomainTestData;
    use crate::non_fungible_domain::import::NonFungibleTokenError;
    use crate::reflector;
    use crate::registry_v2::RegistryV2TestData;
    use crate::registry_v2::import::RegistryV2Errors;
    use crate::utils::{GlobalTestData, create_global_test_data};
    use soroban_sdk::testutils::{BytesN as __, Ledger, MockAuth, MockAuthInvoke};
    use soroban_sdk::{BytesN, Env, IntoVal, vec};

    #[test]
    pub fn test_mint() {
        let e: Env = Env::default();
        let global_test: GlobalTestData = create_global_test_data(&e);
        let nfd_test_data: NonFungibleDomainTestData = NonFungibleDomainTestData::create_test_data(&e, &global_test);

        let node: BytesN<32> = BytesN::random(&e);
        assert!(nfd_test_data.nfd_client.try_mint(&nfd_test_data.owner_1, &1, &node).is_err());

        nfd_test_data
            .nfd_client
            .mock_auths(&[MockAuth {
                address: &global_test.registry_v2,
                invoke: &MockAuthInvoke {
                    contract: &global_test.nfd,
                    fn_name: "mint",
                    args: (nfd_test_data.owner_1.clone(), 1u32, node.clone()).into_val(&e),
                    sub_invokes: &[],
                },
            }])
            .mint(&nfd_test_data.owner_1, &1, &node);

        assert_eq!(nfd_test_data.nfd_client.total_supply(), 1u32);
        assert_eq!(nfd_test_data.nfd_client.balance(&nfd_test_data.owner_1), 1u32);
        assert_eq!(nfd_test_data.nfd_client.owner_of(&1u32), nfd_test_data.owner_1);

        let already_exist_error = nfd_test_data
            .nfd_client
            .mock_all_auths()
            .try_mint(&nfd_test_data.owner_1, &1, &node)
            .unwrap_err()
            .unwrap();

        assert_eq!(already_exist_error, NonFungibleTokenError::TokenIdAlreadyExist.into());
    }

    #[test]
    pub fn test_transfer() {
        let e: Env = Env::default();
        e.ledger().set_timestamp(900);
        let global_test_data: GlobalTestData = create_global_test_data(&e);
        let v2_registry_test_data: RegistryV2TestData = RegistryV2TestData::create_test_data(&e, &global_test_data);
        let nfd_test_data: NonFungibleDomainTestData = NonFungibleDomainTestData::create_test_data(&e, &global_test_data);

        let (oracle_client, _) = reflector::init_reflector(&e, &global_test_data.adm, &global_test_data.oracle_addr);
        oracle_client
            .mock_all_auths()
            .set_price(&vec![&e, 2376i128 * 10i128.pow(10)], &600_000);

        global_test_data
            .col_asset_stellar
            .mock_all_auths()
            .mint(&nfd_test_data.owner_1, &100_0000000i128);

        v2_registry_test_data.registry_client.mock_all_auths().register(
            &v2_registry_test_data.domain,
            &v2_registry_test_data.tld,
            &nfd_test_data.owner_1,
            &v2_registry_test_data.address,
            &v2_registry_test_data.periods,
        );

        assert!(
            nfd_test_data
                .nfd_client
                .try_transfer(&nfd_test_data.owner_1, &nfd_test_data.owner_2, &1u32)
                .is_err(),
        );

        nfd_test_data
            .nfd_client
            .mock_auths(&[MockAuth {
                address: &nfd_test_data.owner_1,
                invoke: &MockAuthInvoke {
                    contract: &nfd_test_data.nfd_client.address,
                    fn_name: "transfer",
                    args: (nfd_test_data.owner_1.clone(), nfd_test_data.owner_2.clone(), 1u32).into_val(&e),
                    sub_invokes: &[],
                },
            }])
            .transfer(&nfd_test_data.owner_1, &nfd_test_data.owner_2, &1u32);

        assert_eq!(nfd_test_data.nfd_client.total_supply(), 1u32);
        assert_eq!(nfd_test_data.nfd_client.balance(&nfd_test_data.owner_1), 0u32);
        assert_eq!(nfd_test_data.nfd_client.owner_of(&1u32), nfd_test_data.owner_2);
        assert_eq!(nfd_test_data.nfd_client.balance(&nfd_test_data.owner_2), 1u32);

        let incorrect_owner_error = nfd_test_data
            .nfd_client
            .mock_all_auths()
            .try_transfer(&nfd_test_data.owner_1, &nfd_test_data.owner_2, &1u32)
            .unwrap_err()
            .unwrap();

        assert_eq!(incorrect_owner_error, NonFungibleTokenError::IncorrectOwner.into());

        let non_existing_token = nfd_test_data
            .nfd_client
            .mock_all_auths()
            .try_transfer(&nfd_test_data.owner_1, &nfd_test_data.owner_2, &2u32)
            .unwrap_err()
            .unwrap();

        assert_eq!(non_existing_token, NonFungibleTokenError::NonExistentToken.into());

        e.ledger().set_timestamp(900 + (3600 * 24 * 360 * 5));

        let invalid_domain = nfd_test_data
            .nfd_client
            .mock_all_auths()
            .try_transfer(&nfd_test_data.owner_2, &nfd_test_data.owner_1, &1u32)
            .unwrap_err()
            .unwrap();

        assert_eq!(invalid_domain, RegistryV2Errors::RecordIsExpired.into());
    }
}
