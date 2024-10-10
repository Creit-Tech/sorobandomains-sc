#![no_std]

mod errors;
mod types;
mod utils;

use crate::errors::ContractErrors;
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, BytesN, Env, Symbol};

mod registry {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/registry.wasm"
    );
}

use crate::registry::Domain;
use crate::types::{StorageKeys, Value};
use crate::utils::{
    extend, extend_key, fetch_domain, get_adm, get_key, pay_fee, remove_key, set_key,
};

const LEDGER_DAY: u32 = 17280;

pub trait KeyValueDBTrait {
    fn set_config(
        e: Env,
        adm: Address,
        registry: Address,
        fee: u128,
        currency: Address,
        treasury: Address,
    );
    fn upgrade(e: Env, hash: BytesN<32>);

    fn set(e: Env, node: BytesN<32>, key: Symbol, value: Value);
    fn get(e: Env, node: BytesN<32>, key: Symbol) -> Option<Value>;
    fn remove(e: Env, node: BytesN<32>, key: Symbol);
}

#[contract]
pub struct KeyValueDB;

#[contractimpl]
impl KeyValueDBTrait for KeyValueDB {
    fn set_config(
        e: Env,
        adm: Address,
        registry: Address,
        fee: u128,
        currency: Address,
        treasury: Address,
    ) {
        if let Some(current_adm) = get_adm(&e) {
            current_adm.require_auth();
        }

        e.storage().instance().set(&StorageKeys::Admin, &adm);
        e.storage()
            .instance()
            .set(&StorageKeys::Registry, &registry);
        e.storage().instance().set(&StorageKeys::Fee, &fee);
        e.storage()
            .instance()
            .set(&StorageKeys::Currency, &currency);
        e.storage()
            .instance()
            .set(&StorageKeys::Treasury, &treasury);

        extend(&e);
    }

    fn upgrade(e: Env, hash: BytesN<32>) {
        get_adm(&e).unwrap().require_auth();
        e.deployer().update_current_contract_wasm(hash);
        extend(&e);
    }

    fn set(e: Env, node: BytesN<32>, key: Symbol, value: Value) {
        let domain: Domain = fetch_domain(&e, &node);
        domain.owner.require_auth();
        pay_fee(&e, &domain.owner);
        set_key(&e, &node, &key, &value, &domain.snapshot);
        extend_key(&e, &node, &key);
        extend(&e);
    }

    fn get(e: Env, node: BytesN<32>, key: Symbol) -> Option<Value> {
        extend_key(&e, &node, &key);
        extend(&e);
        match get_key(&e, &node, &key) {
            Some((value, snapshot)) => {
                let domain: Domain = fetch_domain(&e, &node);
                if domain.snapshot != snapshot {
                    panic_with_error!(&e, &ContractErrors::KeyWasInvalidated);
                }
                Some(value)
            }
            None => None,
        }
    }

    fn remove(e: Env, node: BytesN<32>, key: Symbol) {
        let domain: Domain = fetch_domain(&e, &node);
        domain.owner.require_auth();
        remove_key(&e, &node, &key);
        extend(&e);
    }
}
