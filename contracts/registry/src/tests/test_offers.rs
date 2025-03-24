#![cfg(test)]

use crate::errors::ContractErrors;
use crate::storage::offers::{Offer, OffersDataKeys, OffersFunc};
use crate::storage::record::{Domain, Record, RecordKeys};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use crate::utils::records::generate_node;
use num_integer::div_ceil;
use soroban_sdk::testutils::{Address as _, BytesN as __, Ledger, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Bytes, BytesN, Env, IntoVal};
use test_utils::create_env;

struct Users {
    pub initial_user: Address,
    pub initial_buyer: Address,
    pub second_buyer: Address,
}

fn test_offers_start_data(e: &Env, test_data: &TestData) -> (Users, BytesN<32>) {
    let users: Users = Users {
        initial_user: Address::generate(&e),
        initial_buyer: Address::generate(&e),
        second_buyer: Address::generate(&e),
    };

    test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&users.initial_user, &(68_4956840i128 * 2));

    test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&users.initial_buyer, &(68_4956840i128 * 2));

    test_data
        .col_asset_stellar
        .mock_all_auths()
        .mint(&users.second_buyer, &(68_4956840i128 * 2));

    let new_domain: Bytes = Bytes::from_slice(&e, "stellar".as_bytes());
    let tld: Bytes = Bytes::from_slice(&e, "xlm".as_bytes());

    test_data.contract_client.mock_all_auths().set_record(
        &new_domain,
        &tld,
        &users.initial_user,
        &users.initial_user,
        &test_data.min_duration,
    );

    (users, generate_node(&e, &new_domain, &tld))
}

#[test]
pub fn test_set_offer_non_existing_record_error() {
    let e: Env = create_env();
    let test_data: TestData = create_test_data(&e);
    init_contract(&e, &test_data);

    let error = test_data
        .contract_client
        .mock_all_auths()
        .try_set_offer(&Address::generate(&e), &BytesN::random(&e), &0)
        .unwrap_err()
        .unwrap();

    assert_eq!(error, ContractErrors::RecordDoesntExist.into());
}

#[test]
pub fn test_set_new_buy_offer() {
    let e: Env = create_env();
    let test_data: TestData = create_test_data(&e);
    init_contract(&e, &test_data);

    let (users, target_domain) = test_offers_start_data(&e, &test_data);

    assert_eq!(
        68_4956840u128,
        test_data
            .col_asset_client
            .balance(&test_data.contract_client.address) as u128
    );

    let amount: u128 = 68_4956840u128 + 10_0000000;

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &users.initial_buyer,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "set_offer",
                args: (
                    users.initial_buyer.clone(),
                    target_domain.clone(),
                    amount.clone(),
                )
                    .into_val(&e),
                sub_invokes: &[MockAuthInvoke {
                    contract: &test_data.col_asset_client.address,
                    fn_name: "transfer",
                    args: (
                        users.initial_buyer.clone(),
                        test_data.contract_client.address.clone(),
                        amount.clone() as i128,
                    )
                        .into_val(&e),
                    sub_invokes: &[],
                }],
            },
        }])
        .set_offer(&users.initial_buyer, &target_domain, &amount);

    assert_eq!(
        (68_4956840u128 * 2) + 10_0000000,
        test_data
            .col_asset_client
            .balance(&test_data.contract_client.address) as u128
    );

    e.as_contract(&test_data.contract_client.address, || {
        let offer: Offer = e
            ._offers()
            .get(&OffersDataKeys::BuyOffer(target_domain.clone()))
            .unwrap();

        if let Offer::BuyOffer(buy_offer) = offer {
            assert_eq!(buy_offer.buyer, users.initial_buyer);
            assert_eq!(buy_offer.node, target_domain);
            assert_eq!(buy_offer.snapshot, 1742825701);
            assert_eq!(buy_offer.amount, amount);
        } else {
            panic!();
        }
    });

    assert_eq!(
        68_4956840u128 - 10_0000000,
        test_data.col_asset_client.balance(&users.initial_buyer) as u128
    );
}

#[test]
pub fn test_update_buy_offer_from_same_buyer() {
    let e: Env = create_env();
    let test_data: TestData = create_test_data(&e);
    init_contract(&e, &test_data);

    let (users, target_domain) = test_offers_start_data(&e, &test_data);

    let amount: u128 = 68_4956840u128 + 10_0000000;

    test_data.contract_client.mock_all_auths().set_offer(
        &users.initial_buyer,
        &target_domain,
        &amount,
    );

    let invalid_offer_amount_error = test_data
        .contract_client
        .mock_all_auths()
        .try_set_offer(&users.initial_buyer, &target_domain, &amount)
        .unwrap_err()
        .unwrap();

    assert_eq!(
        invalid_offer_amount_error,
        ContractErrors::InvalidOfferAmount.into(),
    );

    test_data.contract_client.mock_all_auths().set_offer(
        &users.initial_buyer,
        &target_domain,
        &(amount + 10_0000000),
    );

    assert_eq!(
        (68_4956840u128 * 2) + 10_0000000 + 10_0000000,
        test_data
            .col_asset_client
            .balance(&test_data.contract_client.address) as u128
    );

    e.as_contract(&test_data.contract_client.address, || {
        let offer: Offer = e
            ._offers()
            .get(&OffersDataKeys::BuyOffer(target_domain.clone()))
            .unwrap();

        if let Offer::BuyOffer(buy_offer) = offer {
            assert_eq!(buy_offer.buyer, users.initial_buyer);
            assert_eq!(buy_offer.node, target_domain);
            assert_eq!(buy_offer.snapshot, 1742825701);
            assert_eq!(buy_offer.amount, amount + 10_0000000);
        } else {
            panic!();
        }
    });

    assert_eq!(
        68_4956840u128 - 10_0000000 - 10_0000000,
        test_data.col_asset_client.balance(&users.initial_buyer) as u128
    );
}

#[test]
pub fn test_update_buy_offer_from_another_buyer() {
    let e: Env = create_env();
    let test_data: TestData = create_test_data(&e);
    init_contract(&e, &test_data);

    let (users, target_domain) = test_offers_start_data(&e, &test_data);

    let amount: u128 = 68_4956840u128 + 10_0000000;

    test_data.contract_client.mock_all_auths().set_offer(
        &users.initial_buyer,
        &target_domain,
        &amount,
    );

    let invalid_offer_amount_error = test_data
        .contract_client
        .mock_all_auths()
        .try_set_offer(&users.second_buyer, &target_domain, &amount)
        .unwrap_err()
        .unwrap();

    assert_eq!(
        invalid_offer_amount_error,
        ContractErrors::InvalidOfferAmount.into(),
    );

    test_data.contract_client.mock_all_auths().set_offer(
        &users.second_buyer,
        &target_domain,
        &(amount + 10_0000000),
    );

    assert_eq!(
        (68_4956840u128 * 2) + 10_0000000 + 10_0000000,
        test_data
            .col_asset_client
            .balance(&test_data.contract_client.address) as u128
    );

    e.as_contract(&test_data.contract_client.address, || {
        let offer: Offer = e
            ._offers()
            .get(&OffersDataKeys::BuyOffer(target_domain.clone()))
            .unwrap();

        if let Offer::BuyOffer(buy_offer) = offer {
            assert_eq!(buy_offer.buyer, users.second_buyer);
            assert_eq!(buy_offer.node, target_domain);
            assert_eq!(buy_offer.snapshot, 1742825701);
            assert_eq!(buy_offer.amount, amount + 10_0000000);
        } else {
            panic!();
        }
    });

    assert_eq!(
        68_4956840u128 * 2,
        test_data.col_asset_client.balance(&users.initial_buyer) as u128
    );

    assert_eq!(
        68_4956840u128 - 10_0000000 - 10_0000000,
        test_data.col_asset_client.balance(&users.second_buyer) as u128
    );
}

#[test]
pub fn test_set_sale_offer() {
    let e: Env = create_env();
    let test_data: TestData = create_test_data(&e);
    init_contract(&e, &test_data);

    let (users, target_domain) = test_offers_start_data(&e, &test_data);

    let amount: u128 = 68_4956840u128 + 10_0000000;

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &users.initial_user,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "set_offer",
                args: (
                    users.initial_user.clone(),
                    target_domain.clone(),
                    amount.clone(),
                )
                    .into_val(&e),
                sub_invokes: &[],
            },
        }])
        .set_offer(&users.initial_user, &target_domain, &amount);

    // Because is a sale offer, there is no extra collateral in the contract
    assert_eq!(
        68_4956840u128,
        test_data
            .col_asset_client
            .balance(&test_data.contract_client.address) as u128
    );

    e.as_contract(&test_data.contract_client.address, || {
        let offer: Offer = e
            ._offers()
            .get(&OffersDataKeys::SaleOffer(target_domain.clone()))
            .unwrap();

        if let Offer::SaleOffer(sale_offer) = offer {
            assert_eq!(sale_offer.node, target_domain);
            assert_eq!(sale_offer.snapshot, 1742825701);
            assert_eq!(sale_offer.amount, amount);
        } else {
            panic!();
        }
    });

    test_data.contract_client.mock_all_auths().set_offer(
        &users.initial_user,
        &target_domain,
        &(amount + 10_0000000),
    );

    e.as_contract(&test_data.contract_client.address, || {
        let offer: Offer = e
            ._offers()
            .get(&OffersDataKeys::SaleOffer(target_domain.clone()))
            .unwrap();

        if let Offer::SaleOffer(sale_offer) = offer {
            assert_eq!(sale_offer.amount, amount + 10_0000000);
        } else {
            panic!();
        }
    });
}

#[test]
pub fn test_take_sale_offer() {
    let e: Env = create_env();
    let test_data: TestData = create_test_data(&e);
    init_contract(&e, &test_data);

    let (users, target_domain) = test_offers_start_data(&e, &test_data);

    let amount: u128 = 68_4956840u128 + 10_0000000;

    test_data.contract_client.mock_all_auths().set_offer(
        &users.initial_user,
        &target_domain,
        &amount,
    );

    let profit: u128 = amount - 68_4956840u128;
    let fee: u128 = div_ceil(profit * test_data.offer_fee, 100_0000000);

    let contract_balance_before_sale: i128 = test_data
        .col_asset_client
        .balance(&test_data.contract_client.address);

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &users.initial_buyer,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "take_offer",
                args: (users.initial_buyer.clone(), target_domain.clone()).into_val(&e),
                sub_invokes: &[
                    MockAuthInvoke {
                        contract: &test_data.col_asset_client.address,
                        fn_name: "transfer",
                        args: (
                            users.initial_buyer.clone(),
                            users.initial_user.clone(),
                            (amount - fee) as i128,
                        )
                            .into_val(&e),
                        sub_invokes: &[],
                    },
                    MockAuthInvoke {
                        contract: &test_data.col_asset_client.address,
                        fn_name: "transfer",
                        args: (
                            users.initial_buyer.clone(),
                            test_data.fee_taker.clone(),
                            fee as i128,
                        )
                            .into_val(&e),
                        sub_invokes: &[],
                    },
                ],
            },
        }])
        .take_offer(&users.initial_buyer, &target_domain);

    let contract_balance_after_sale: i128 = test_data
        .col_asset_client
        .balance(&test_data.contract_client.address);

    let record: Record = test_data
        .contract_client
        .record(&RecordKeys::Record(target_domain.clone()))
        .unwrap();
    let domain: Domain = match record.clone() {
        Record::Domain(value) => value,
        Record::SubDomain(_) => panic!(),
    };

    assert_eq!(&domain.owner, &users.initial_buyer);
    assert_eq!(&domain.address, &users.initial_buyer);

    e.as_contract(&test_data.contract_client.address, || {
        assert!(e
            ._offers()
            .get(&OffersDataKeys::SaleOffer(target_domain.clone()))
            .is_none());
    });

    assert_eq!(contract_balance_before_sale, contract_balance_after_sale);

    // The seller should have the initial collateral back plus the profit minus protocol fee
    assert_eq!(
        (68_4956840u128 * 2) + (10_0000000 - fee),
        test_data.col_asset_client.balance(&users.initial_user) as u128
    );

    assert_eq!(
        fee,
        test_data.col_asset_client.balance(&test_data.fee_taker) as u128
    );
}

#[test]
pub fn test_take_buy_offer() {
    let e: Env = create_env();
    let test_data: TestData = create_test_data(&e);
    init_contract(&e, &test_data);

    let (users, target_domain) = test_offers_start_data(&e, &test_data);

    let amount: u128 = 68_4956840u128 + 10_0000000;

    let contract_balance_before_sale: i128 = test_data
        .col_asset_client
        .balance(&test_data.contract_client.address);

    test_data.contract_client.mock_all_auths().set_offer(
        &users.initial_buyer,
        &target_domain,
        &amount,
    );

    let profit: u128 = amount - 68_4956840u128;
    let fee: u128 = div_ceil(profit * test_data.offer_fee, 100_0000000);

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &users.initial_user,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "take_offer",
                args: (users.initial_user.clone(), target_domain.clone()).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .take_offer(&users.initial_user, &target_domain);

    let contract_balance_after_sale: i128 = test_data
        .col_asset_client
        .balance(&test_data.contract_client.address);

    let record: Record = test_data
        .contract_client
        .record(&RecordKeys::Record(target_domain.clone()))
        .unwrap();
    let domain: Domain = match record.clone() {
        Record::Domain(value) => value,
        Record::SubDomain(_) => panic!(),
    };

    assert_eq!(&domain.owner, &users.initial_buyer);
    assert_eq!(&domain.address, &users.initial_buyer);

    e.as_contract(&test_data.contract_client.address, || {
        assert!(e
            ._offers()
            .get(&OffersDataKeys::BuyOffer(target_domain.clone()))
            .is_none());
    });

    assert_eq!(contract_balance_before_sale, contract_balance_after_sale);

    // The seller should have the initial collateral back plus the profit minus protocol fee
    assert_eq!(
        (68_4956840u128 * 2) + (10_0000000 - fee),
        test_data.col_asset_client.balance(&users.initial_user) as u128
    );

    assert_eq!(
        fee,
        test_data.col_asset_client.balance(&test_data.fee_taker) as u128
    );
}

#[test]
pub fn test_burn_offers() {
    let e: Env = create_env();
    let test_data: TestData = create_test_data(&e);
    init_contract(&e, &test_data);

    let (users, target_domain) = test_offers_start_data(&e, &test_data);

    let amount: u128 = 68_4956840u128 + 10_0000000;

    test_data.contract_client.mock_all_auths().set_offer(
        &users.initial_user,
        &target_domain,
        &amount,
    );

    test_data.contract_client.mock_all_auths().set_offer(
        &users.initial_buyer,
        &target_domain,
        &amount,
    );

    e.as_contract(&test_data.contract_client.address, || {
        assert!(e
            ._offers()
            .get(&OffersDataKeys::SaleOffer(target_domain.clone()))
            .is_some());

        assert!(e
            ._offers()
            .get(&OffersDataKeys::BuyOffer(target_domain.clone()))
            .is_some());
    });

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &users.initial_user,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "burn_offer",
                args: (OffersDataKeys::SaleOffer(target_domain.clone()),).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .burn_offer(&OffersDataKeys::SaleOffer(target_domain.clone()));

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &users.initial_buyer,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "burn_offer",
                args: (OffersDataKeys::BuyOffer(target_domain.clone()),).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .burn_offer(&OffersDataKeys::BuyOffer(target_domain.clone()));

    e.as_contract(&test_data.contract_client.address, || {
        assert!(e
            ._offers()
            .get(&OffersDataKeys::SaleOffer(target_domain.clone()))
            .is_none());

        assert!(e
            ._offers()
            .get(&OffersDataKeys::BuyOffer(target_domain.clone()))
            .is_none());
    });

    assert_eq!(
        (68_4956840u128) * 2,
        test_data.col_asset_client.balance(&users.initial_buyer) as u128
    );
}

#[test]
pub fn test_take_offer_errors() {
    // todo!()
}
