use crate::errors::ContractErrors;
use crate::registry::{ContractErrors as RegistryErrors, Domain, Record, RecordKeys};
use crate::{StorageKeys, Value, LEDGER_DAY};
use soroban_sdk::{panic_with_error, symbol_short, token, Address, BytesN, Env, IntoVal, Symbol};

pub fn set_key(e: &Env, node: &BytesN<32>, key: &Symbol, value: &Value, snapshot: &u64) {
    e.storage().persistent().set::<StorageKeys, (Value, u64)>(
        &StorageKeys::Key((node.clone(), key.clone())),
        &(value.clone(), snapshot.clone()),
    );
}

pub fn get_key(e: &Env, node: &BytesN<32>, key: &Symbol) -> Option<(Value, u64)> {
    e.storage()
        .persistent()
        .get(&StorageKeys::Key((node.clone(), key.clone())))
}

pub fn extend_key(e: &Env, node: &BytesN<32>, key: &Symbol) {
    e.storage().persistent().extend_ttl(
        &StorageKeys::Key((node.clone(), key.clone())),
        LEDGER_DAY * 45,
        LEDGER_DAY * 90,
    );
}

pub fn remove_key(e: &Env, node: &BytesN<32>, key: &Symbol) {
    e.storage()
        .persistent()
        .remove(&StorageKeys::Key((node.clone(), key.clone())));
}

pub fn pay_fee(e: &Env, caller: &Address) {
    let treasury: Address = e.storage().instance().get(&StorageKeys::Treasury).unwrap();
    let currency: Address = e.storage().instance().get(&StorageKeys::Currency).unwrap();
    let fee: u128 = e.storage().instance().get(&StorageKeys::Fee).unwrap();

    let result = token::Client::new(&e, &currency).try_transfer(&caller, &treasury, &(fee as i128));

    if result.is_err() {
        panic_with_error!(&e, &ContractErrors::FailedToGetRecord);
    }
}

pub fn get_adm(e: &Env) -> Option<Address> {
    e.storage().instance().get(&StorageKeys::Admin)
}

pub fn fetch_domain(e: &Env, node: &BytesN<32>) -> Domain {
    let registry = e.storage().instance().get(&StorageKeys::Registry).unwrap();
    let result = e.try_invoke_contract::<Option<Record>, RegistryErrors>(
        &registry,
        &symbol_short!("record"),
        (RecordKeys::Record(node.clone()),).into_val(e),
    );

    if result.is_err() {
        panic_with_error!(&e, &ContractErrors::FailedToGetRecord);
    }

    let record: Record = result.unwrap().unwrap().unwrap_or_else(|| {
        panic_with_error!(&e, &ContractErrors::FailedToGetRecord);
    });

    match record {
        Record::Domain(value) => value,
        Record::SubDomain(_) => {
            // We shouldn't be able to reach this case but we panic just in case.
            panic_with_error!(&e, &ContractErrors::FailedToGetRecord);
        }
    }
}

pub fn extend(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_DAY * 30, LEDGER_DAY * 60);
}
