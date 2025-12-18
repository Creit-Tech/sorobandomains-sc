use common::constants::{LEDGER_MONTH, LEDGER_WEEK};
use common::storage::{instance_consumer, persistent_consumer};
use soroban_sdk::{contracttype, Address, BytesN, Env, String};

#[contracttype]
pub struct ApprovalData {
    pub approved: Address,
    pub live_until_ledger: u32,
}

#[contracttype]
pub struct Metadata {
    pub base_uri: String,
    pub name: String,
    pub symbol: String,
}

#[contracttype]
pub enum DomainStorageKeys {
    Owner(u32),                       // -> Address
    Balance(Address),                 // -> u32
    Approval(u32),                    // -> ApprovalData
    Metadata,                         // -> Metadata
    ApprovalForAll(Address, Address), // -> u32 (live_until_ledger)
    Registry,                         // -> Address
    Admin,                            // -> Address
    TotalSupply,                      // -> u32
    TokenNode(u32),                   // -> BytesN<32>
}

pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(LEDGER_WEEK, LEDGER_MONTH);
}

pub fn owner(e: &Env, token_id: &u32, value: Option<Address>) -> Option<Address> {
    persistent_consumer(&e, DomainStorageKeys::Owner(token_id.clone()), value)
}

pub fn balance(e: &Env, account: &Address, value: Option<u32>) -> Option<u32> {
    persistent_consumer(&e, DomainStorageKeys::Balance(account.clone()), value)
}

pub fn approval(e: &Env, token_id: &u32, value: Option<ApprovalData>) -> Option<ApprovalData> {
    let key = DomainStorageKeys::Approval(token_id.clone());
    if let Some(v) = value {
        e.storage().temporary().set(&key, &v);
        let live_for: u32 = v.live_until_ledger - e.ledger().sequence();
        e.storage().temporary().extend_ttl(&key, live_for, live_for);
    }
    e.storage().temporary().get(&key)
}

pub fn approval_for_all(e: &Env, owner: &Address, operator: &Address, value: Option<u32>) -> Option<u32> {
    let key = DomainStorageKeys::ApprovalForAll(owner.clone(), operator.clone());
    if let Some(v) = value {
        e.storage().temporary().set(&key, &v);
        let live_for: u32 = v - e.ledger().sequence();
        e.storage().temporary().extend_ttl(&key, live_for, live_for);
    }
    e.storage().temporary().get(&key)
}

pub fn metadata(e: &Env, value: Option<Metadata>) -> Option<Metadata> {
    instance_consumer(&e, DomainStorageKeys::Metadata, value)
}

pub fn registry(e: &Env, value: Option<Address>) -> Option<Address> {
    instance_consumer(&e, DomainStorageKeys::Registry, value)
}

pub fn admin(e: &Env, value: Option<Address>) -> Option<Address> {
    instance_consumer(&e, DomainStorageKeys::Admin, value)
}

pub fn total_supply(e: &Env, value: Option<u32>) -> Option<u32> {
    instance_consumer(&e, DomainStorageKeys::TotalSupply, value)
}

pub fn token_node(e: &Env, token_id: &u32, value: Option<BytesN<32>>) -> Option<BytesN<32>> {
    persistent_consumer(&e, DomainStorageKeys::TokenNode(token_id.clone()), value)
}

// ---- State updaters

pub fn remove_approval(e: &Env, token_id: &u32) {
    e.storage().temporary().remove(&DomainStorageKeys::Approval(token_id.clone()));
}

pub fn remove_approval_for_all(e: &Env, owner: &Address, operator: &Address) {
    e.storage()
        .temporary()
        .remove(&DomainStorageKeys::ApprovalForAll(owner.clone(), operator.clone()));
}

pub fn burn_token(e: &Env, token_id: &u32) {
    e.storage().persistent().remove(&DomainStorageKeys::Owner(token_id.clone()));
    e.storage().persistent().remove(&DomainStorageKeys::TokenNode(token_id.clone()));
}
