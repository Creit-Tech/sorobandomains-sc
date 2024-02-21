use crate::errors::ContractErrors;
use crate::storage::core::{CoreData, CoreDataEntity};
use crate::storage::record::{Record, RecordEntity, SubRecord};
use crate::utils::records::{generate_node, validate_domain};
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
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);

    fn core_data(e: Env) -> Option<CoreData>;

    fn set_record(
        e: Env,
        domain: Bytes,
        tld: Bytes,
        owner: Address,
        address: Address,
        duration: u64,
    );

    fn set_sub(e: Env, sub: Bytes, parent: BytesN<32>, address: Address);

    /// Get a record based on the node hash
    fn record(e: Env, node: BytesN<32>) -> Option<Record>;

    fn parse_domain(e: Env, domain: Bytes, tld: Bytes) -> BytesN<32>;

    /// When burning a record, the record gets removed from the storage and the collateral is released
    fn burn_record(e: Env, node: BytesN<32>);
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

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        e.bump_core();
        e.is_adm();
        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn core_data(e: Env) -> Option<CoreData> {
        e.bump_core();
        e.core_data()
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

        // We check if the record already exists, if it does then we panic
        if e.record(&node_hash).is_some() {
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

        e.set_record(&Record {
            node: node_hash.clone(),
            owner,
            address,
            exp_date,
            collateral,
        });

        // TODO: add an event

        e.bump_record(&node_hash);
    }

    fn set_sub(e: Env, sub: Bytes, parent: BytesN<32>, address: Address) {
        e.bump_core();

        validate_domain(&e, &sub);

        let parent_record: Record = e
            .record(&parent)
            .unwrap_or_else(|| panic_with_error!(&e, &ContractErrors::ParentDoesntExist));

        parent_record.owner.require_auth();

        if parent_record.exp_date < e.ledger().timestamp() {
            panic_with_error!(&e, &ContractErrors::ExpiredDomain);
        }

        let node_hash: BytesN<32> = generate_node(&e, &sub, &(Bytes::from(parent.clone())));

        e.set_sub(&SubRecord {
            node: node_hash.clone(),
            parent,
            address,
        });

        e.bump_sub(&node_hash);
    }

    fn record(e: Env, node: BytesN<32>) -> Option<Record> {
        e.bump_core();

        if let Some(record) = e.record(&node) {
            e.bump_record(&node);

            if record.exp_date < e.ledger().timestamp() {
                panic_with_error!(&e, &ContractErrors::ExpiredDomain);
            }

            Some(record)
        } else {
            None
        }
    }

    fn parse_domain(e: Env, domain: Bytes, tld: Bytes) -> BytesN<32> {
        e.bump_core();
        generate_node(&e, &domain, &tld)
    }

    fn burn_record(e: Env, node: BytesN<32>) {
        e.bump_core();
        let core_data: CoreData = e.core_data().unwrap();
        let record: Record = match e.record(&node) {
            Some(record) => record,
            None => panic_with_error!(&e, ContractErrors::RecordDoesntExist),
        };

        record.owner.require_auth();

        e.burn_record(&node);

        token::Client::new(&e, &core_data.col_asset).transfer(
            &e.current_contract_address(),
            &record.owner,
            &(record.collateral as i128),
        );

        // TODO: Add event
    }
}
