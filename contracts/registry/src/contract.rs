use crate::errors::ContractErrors;
use crate::events::emit_offer_accepted;
use crate::storage::core::{CoreData, CoreDataEntity, OffersConfig};
use crate::storage::offers::{Offer, OffersDataKeys, OffersFunc};
use crate::storage::record::{Domain, Record, RecordEntity, RecordKeys, SubDomain};
use crate::utils::offers::{set_new_buy_offer, set_sale_offer, update_buy_offer};
use crate::utils::records::{generate_node, validate_domain};
use num_integer::div_ceil;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, token, Address, Bytes, BytesN, Env, Vec,
};

pub trait RegistryContractTrait {
    fn init(
        e: Env,
        adm: Address,
        node_rate: u128,
        col_asset: Address,
        min_duration: u64,
        allowed_tlds: Vec<Bytes>,
    );

    fn set_offers_config(e: Env, fee_taker: Address, fee: u128);

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);
    fn update_tlds(e: Env, tlds: Vec<Bytes>);

    fn set_record(
        e: Env,
        domain: Bytes,
        tld: Bytes,
        owner: Address,
        address: Address,
        duration: u64,
    );

    fn update_address(e: Env, key: RecordKeys, address: Address);

    fn set_sub(e: Env, sub: Bytes, parent: RecordKeys, address: Address);

    // Get a record based on the node hash
    fn record(e: Env, key: RecordKeys) -> Option<Record>;

    fn parse_domain(e: Env, domain: Bytes, tld: Bytes) -> BytesN<32>;

    // The owner of a domain can transfer it to a different address
    // This method also invalidates all the subdomains, this is just for prevention purposes but this can be changed in the future if people think there is no risk on it.
    fn transfer(e: Env, key: RecordKeys, to: Address);

    // When burning a record, the record gets removed from the storage and the collateral is released
    fn burn_record(e: Env, key: RecordKeys);

    // Users can set a domain for sale or set a buy offer for that domain
    // Users set the offer amount and if is a buy offer the contract takes the amount and keep it in case the owner accepts it
    // If the offer to set is a BuyOffer and there is already one set we check if the amount is higher,
    // if is not higher we reject it and if is higher we return the amount to the old offer owner, and we take the new one.
    // If is a BuyOffer, the amount needs to be higher than the domain collateral
    fn set_offer(e: Env, caller: Address, node: BytesN<32>, amount: u128);

    // An offer needs to be valid (same snapshot number) in order to be accepted
    // If is a SaleOffer, the domain is transferred to the "caller" and the "caller" transfer the funds to the old owner of the domain
    // If is a BuyOffer, the amount is sent to the old owner of the domain from the contract and the domain is transferred to the "buyer" in the Offer
    // The protocol fee is taken at the moment of the transfer, if there is no fee then the default is 3%
    fn take_offer(e: Env, caller: Address, node: BytesN<32>);

    fn burn_offer(e: Env, key: OffersDataKeys);
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContractTrait for RegistryContract {
    fn init(
        e: Env,
        adm: Address,
        node_rate: u128,
        col_asset: Address,
        min_duration: u64,
        allowed_tlds: Vec<Bytes>,
    ) {
        if let Some(_) = e.core_data() {
            panic_with_error!(&e, &ContractErrors::AlreadyStarted);
        } else {
            e.set_core_data(&CoreData {
                adm,
                node_rate,
                col_asset,
                min_duration,
                allowed_tlds,
            });
            e.bump_core();
        }
    }

    fn set_offers_config(e: Env, fee_taker: Address, fee: u128) {
        e.bump_core();
        e.is_adm();
        e.set_offers_config(&OffersConfig { fee_taker, fee });
    }

    fn upgrade(e: Env, hash: BytesN<32>) {
        e.bump_core();
        e.is_adm();
        e.deployer().update_current_contract_wasm(hash);
    }

    fn update_tlds(e: Env, tlds: Vec<Bytes>) {
        e.bump_core();
        e.is_adm();
        let mut core: CoreData = e.core_data().unwrap();
        core.allowed_tlds = tlds;
        e.set_core_data(&core);
    }

    fn set_record(
        e: Env,
        domain: Bytes,
        tld: Bytes,
        owner: Address,
        address: Address,
        duration: u64,
    ) {
        e.bump_core();
        owner.require_auth();

        validate_domain(&e, &domain);

        let core_data: CoreData = e.core_data().unwrap();

        if !core_data.allowed_tlds.contains(tld.clone()) {
            panic_with_error!(&e, &ContractErrors::UnsupportedTLD);
        }

        let node_hash: BytesN<32> = generate_node(&e, &domain, &tld);
        let record_key: RecordKeys = RecordKeys::Record(node_hash.clone());

        // We check if the record already exists, if it does then we panic
        if e.record(&record_key).is_some() {
            panic_with_error!(&e, &ContractErrors::RecordAlreadyExist);
        }

        if duration < core_data.min_duration {
            panic_with_error!(&e, &ContractErrors::InvalidDuration);
        }

        let exp_date: u64 = e.ledger().timestamp() + duration;
        let multiplier: u32 = if domain.len() > 4 {
            1
        } else {
            (5 - domain.len()) * 3
        };

        let collateral: u128 = core_data.node_rate * (duration as u128) * (multiplier as u128);

        token::Client::new(&e, &core_data.col_asset).transfer(
            &owner,
            &e.current_contract_address(),
            &(collateral as i128),
        );

        e.set_record(&Record::Domain(Domain {
            node: node_hash,
            owner,
            address,
            exp_date,
            collateral,
            snapshot: e.ledger().timestamp(),
        }));

        // TODO: add an event

        e.bump_record(&record_key);
    }

    fn update_address(e: Env, key: RecordKeys, address: Address) {
        e.bump_core();
        let record: Record = match e.record(&key) {
            Some(record) => record,
            None => panic_with_error!(&e, ContractErrors::RecordDoesntExist),
        };

        if let Record::Domain(mut domain) = record {
            domain.owner.require_auth();
            domain.address = address;
            e.set_record(&Record::Domain(domain));
            e.bump_record(&key);
        } else {
            panic_with_error!(&e, ContractErrors::InvalidParent);
        }
    }

    fn set_sub(e: Env, sub: Bytes, parent: RecordKeys, address: Address) {
        e.bump_core();

        validate_domain(&e, &sub);

        let parent_record: Record = e
            .record(&parent)
            .unwrap_or_else(|| panic_with_error!(&e, &ContractErrors::InvalidParent));

        if let Record::Domain(domain) = parent_record {
            domain.owner.require_auth();

            if domain.exp_date < e.ledger().timestamp() {
                panic_with_error!(&e, &ContractErrors::ExpiredDomain);
            }

            let node_hash: BytesN<32> =
                generate_node(&e, &sub, &(Bytes::from(domain.node.clone())));
            let record_key: RecordKeys = RecordKeys::SubRecord(node_hash.clone());

            e.set_record(&Record::SubDomain(SubDomain {
                node: node_hash,
                parent: domain.node.clone(),
                address,
                snapshot: domain.snapshot,
            }));

            e.bump_record(&record_key);
        } else {
            panic_with_error!(&e, &ContractErrors::InvalidParent)
        }
    }

    fn record(e: Env, key: RecordKeys) -> Option<Record> {
        e.bump_core();

        let record: Option<Record> = e.record(&key);

        if record.is_none() {
            return None;
        }

        match record.unwrap() {
            Record::Domain(domain) => {
                if domain.exp_date < e.ledger().timestamp() {
                    panic_with_error!(&e, &ContractErrors::ExpiredDomain);
                }

                Some(Record::Domain(domain))
            }
            Record::SubDomain(sub) => {
                if let Record::Domain(domain) =
                    e.record(&RecordKeys::Record(sub.parent.clone())).unwrap()
                {
                    if domain.exp_date < e.ledger().timestamp() {
                        panic_with_error!(&e, &ContractErrors::ExpiredDomain);
                    }

                    if domain.snapshot != sub.snapshot {
                        panic_with_error!(&e, &ContractErrors::OutdatedSub);
                    }
                } else {
                    panic_with_error!(&e, &ContractErrors::InvalidParent);
                }

                Some(Record::SubDomain(sub))
            }
        }
    }

    fn parse_domain(e: Env, domain: Bytes, tld: Bytes) -> BytesN<32> {
        e.bump_core();
        generate_node(&e, &domain, &tld)
    }

    fn transfer(e: Env, key: RecordKeys, to: Address) {
        e.bump_core();
        let record: Record = match e.record(&key) {
            Some(record) => record,
            None => panic_with_error!(&e, ContractErrors::RecordDoesntExist),
        };

        if let Record::Domain(mut domain) = record {
            domain.owner.require_auth();
            domain.owner = to;
            domain.snapshot = e.ledger().timestamp();
            e.set_record(&Record::Domain(domain));
            e.bump_record(&key);
        } else {
            panic_with_error!(&e, ContractErrors::InvalidTransfer);
        }
    }

    fn burn_record(e: Env, key: RecordKeys) {
        e.bump_core();
        let core_data: CoreData = e.core_data().unwrap();
        let record: Record = match e.record(&key) {
            Some(record) => record,
            None => panic_with_error!(&e, ContractErrors::RecordDoesntExist),
        };

        match record {
            Record::Domain(domain) => {
                domain.owner.require_auth();
                e.burn_record(&RecordKeys::Record(domain.node.clone()));
                token::Client::new(&e, &core_data.col_asset).transfer(
                    &e.current_contract_address(),
                    &domain.owner,
                    &(domain.collateral as i128),
                );
            }
            Record::SubDomain(sub) => {
                if let Record::Domain(domain) =
                    e.record(&RecordKeys::Record(sub.parent.clone())).unwrap()
                {
                    domain.owner.require_auth();
                } else {
                    panic_with_error!(&e, &ContractErrors::InvalidParent);
                }
                e.burn_record(&RecordKeys::Record(sub.node.clone()));
            }
        }

        // TODO: Add event
    }

    fn set_offer(e: Env, caller: Address, node: BytesN<32>, amount: u128) {
        e.bump_core();
        caller.require_auth();

        let record: Record = match e.record(&RecordKeys::Record(node.clone())) {
            Some(record) => record,
            None => panic_with_error!(&e, ContractErrors::RecordDoesntExist),
        };

        e.bump_record(&RecordKeys::Record(node.clone()));

        let domain: Domain = match record {
            Record::Domain(v) => v,
            // Should be impossible to reach this, but we panic just in case
            Record::SubDomain(_) => panic_with_error!(&e, ContractErrors::InvalidDomain),
        };

        let is_sale: bool = domain.owner == caller;

        if !is_sale && amount <= domain.collateral {
            panic_with_error!(&e, &ContractErrors::InvalidOfferAmount);
        }

        let offer_key: OffersDataKeys = if is_sale {
            OffersDataKeys::SaleOffer(node.clone())
        } else {
            OffersDataKeys::BuyOffer(node.clone())
        };
        let offer: Option<Offer> = e._offers().get(&offer_key);

        match offer {
            None => {
                if is_sale {
                    set_sale_offer(&e, &domain, &amount);
                } else {
                    set_new_buy_offer(&e, &e.core_data().unwrap(), &caller, &domain, &amount);
                }
            }
            Some(old_offer) => match old_offer {
                Offer::BuyOffer(old_buy_offer) => {
                    // We shouldn't be able to pass this condition but just in case we panic.
                    if is_sale {
                        panic_with_error!(&e, &ContractErrors::UnexpectedError);
                    }

                    update_buy_offer(
                        &e,
                        &e.core_data().unwrap(),
                        &caller,
                        &old_buy_offer,
                        &domain,
                        &amount,
                    );
                }
                Offer::SaleOffer(_) => {
                    set_sale_offer(&e, &domain, &amount);
                }
            },
        }
    }

    fn take_offer(e: Env, caller: Address, node: BytesN<32>) {
        e.bump_core();
        caller.require_auth();

        let mut domain: Domain = match e.record(&RecordKeys::Record(node.clone())).unwrap() {
            Record::Domain(d) => d,
            Record::SubDomain(_) => panic_with_error!(&e, &ContractErrors::InvalidDomain),
        };

        // In this case the condition is the other way around because the seller (domain owner)
        // is accepting a buy Offer instead of making its own sale offer
        let is_seller: bool = domain.owner != caller;

        let offer_key: OffersDataKeys = if is_seller {
            OffersDataKeys::SaleOffer(node.clone())
        } else {
            OffersDataKeys::BuyOffer(node.clone())
        };

        let offer: Offer = e._offers().get(&offer_key).unwrap();
        let core_data: CoreData = e.core_data().unwrap();
        let offers_config: OffersConfig = e.offers_config().unwrap();

        match offer {
            Offer::BuyOffer(buy_offer) => {
                if domain.snapshot != buy_offer.snapshot {
                    panic_with_error!(&e, &ContractErrors::OutdatedOffer);
                }

                let profit: u128 = buy_offer.amount - domain.collateral;
                // TODO: Test minimal fee (for example a profit of 0_0000001)
                let fee: u128 = div_ceil(profit * offers_config.fee, 100_0000000);

                token::Client::new(&e, &core_data.col_asset).transfer(
                    &e.current_contract_address(),
                    &domain.owner,
                    &((buy_offer.amount - fee) as i128),
                );

                token::Client::new(&e, &core_data.col_asset).transfer(
                    &e.current_contract_address(),
                    &offers_config.fee_taker,
                    &(fee as i128),
                );

                emit_offer_accepted(
                    &e,
                    &buy_offer.buyer,
                    &domain.owner,
                    &domain.node,
                    &buy_offer.amount,
                );

                domain.owner = buy_offer.buyer.clone();
                domain.address = buy_offer.buyer;
                domain.snapshot = e.ledger().timestamp();
                e.set_record(&Record::Domain(domain));
                e._offers().burn(&OffersDataKeys::BuyOffer(buy_offer.node));
            }
            Offer::SaleOffer(sale_offer) => {
                if domain.snapshot != sale_offer.snapshot {
                    panic_with_error!(&e, &ContractErrors::OutdatedOffer);
                }

                let profit: u128 = sale_offer.amount - domain.collateral;
                let fee: u128 = div_ceil(profit * offers_config.fee, 100_0000000);

                token::Client::new(&e, &core_data.col_asset).transfer(
                    &caller,
                    &domain.owner,
                    &((sale_offer.amount - fee) as i128),
                );

                token::Client::new(&e, &core_data.col_asset).transfer(
                    &caller,
                    &offers_config.fee_taker,
                    &(fee as i128),
                );

                emit_offer_accepted(&e, &caller, &domain.owner, &domain.node, &sale_offer.amount);

                domain.owner = caller.clone();
                domain.address = caller;
                domain.snapshot = e.ledger().timestamp();
                e.set_record(&Record::Domain(domain));
                e._offers()
                    .burn(&OffersDataKeys::SaleOffer(sale_offer.node));
            }
        }

        e.bump_record(&RecordKeys::Record(node.clone()));
    }

    fn burn_offer(e: Env, key: OffersDataKeys) {
        e.bump_core();
        let offer: Offer = e._offers().get(&key).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::OfferDoesntExist);
        });

        match offer {
            Offer::BuyOffer(buy_offer) => {
                buy_offer.buyer.require_auth();

                token::Client::new(&e, &e.core_data().unwrap().col_asset).transfer(
                    &e.current_contract_address(),
                    &buy_offer.buyer,
                    &(buy_offer.amount as i128),
                );

                e._offers().burn(&key);
            }
            Offer::SaleOffer(sale_offer) => {
                let domain: Record = e
                    .record(&RecordKeys::Record(sale_offer.node))
                    .unwrap_or_else(|| {
                        panic_with_error!(&e, &ContractErrors::RecordDoesntExist);
                    });

                match domain {
                    Record::Domain(domain) => {
                        domain.owner.require_auth();
                        e._offers().burn(&key);
                    }
                    Record::SubDomain(_) => panic_with_error!(&e, &ContractErrors::InvalidDomain),
                };
            }
        }
    }
}
