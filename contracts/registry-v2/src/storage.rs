use common::constants::{LEDGER_MONTH, LEDGER_WEEK};
use common::storage::{instance_consumer, persistent_consumer};
use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, Vec};

#[contracttype]
pub enum RegistryStorageKeys {
    Index, // -> This is the current index of the issued domains, it will increase by 1 on each new domain registered
    Admin,
    Oracle,
    PayingAsset,
    TLDs,
    NFD, // -> Address (The address of the "Non-Fungible Token" contract)
    Domain(BytesN<32>),
    SubDomain(BytesN<32>),
    Treasury, // -> Address (this is the address to which the admin can send the funds to, at some point this will be the DAO contract)

    V1Registry,
    V1MaxSnapshot, // -> The max timestamp at which a v1 domain has been updated to be accepted for migration
    V1Deadline,    // -> This is the deadline to migrate domains from the v1 to this registry
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Domain {
    pub domain: Bytes,
    pub tld: Bytes,

    // The `node` is the hash following the logic used by the function `generate_domain_node`
    pub node: BytesN<32>,

    // The NFD token id this domain will hold
    pub token_id: u32,

    // The address is where the node resolves to
    // IMPORTANT: this value can be different to the current owner of the nfd token
    pub address: Address,

    // The TTL is the end expiration date of the domain.
    // A domain that have been expired for at least 30 days can be claimed by another address
    pub exp_date: u64,

    // This value gets updated on each renew of the domain (creation included).
    // This value also serves as the `snapshot` for validating sub records.
    pub snapshot: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct SubDomain {
    pub domain: Bytes,

    // The node is the hash of the subdomain
    pub node: BytesN<32>,

    // Parent is the hash of the node hash above this record
    // The parent could be another subdomain from the same root domain
    pub parent: BytesN<32>,

    // The address is where the node resolves to
    pub address: Address,

    // The node hash of the root domain
    pub root: BytesN<32>,

    // The snapshot is taken from the root domain valid date
    // If the subdomain snapshot is lower that the root valid_date, it means the subdomain is invalid
    pub snapshot: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum RecordKey {
    Domain(BytesN<32>),
    SubDomain(BytesN<32>),
}

pub fn consume_index(e: &Env) -> u32 {
    let current: u32 = instance_consumer(&e, &RegistryStorageKeys::Index, None).unwrap_or(0);
    instance_consumer(&e, RegistryStorageKeys::Index, Some(current + 1)).unwrap()
}

pub fn admin(e: &Env, value: Option<Address>) -> Option<Address> {
    instance_consumer(&e, &RegistryStorageKeys::Admin, value)
}

pub fn oracle(e: &Env, value: Option<Address>) -> Option<Address> {
    instance_consumer(&e, &RegistryStorageKeys::Oracle, value)
}

pub fn paying_asset(e: &Env, value: Option<Address>) -> Option<Address> {
    instance_consumer(&e, &RegistryStorageKeys::PayingAsset, value)
}

pub fn tlds(e: &Env, value: Option<Vec<Bytes>>) -> Option<Vec<Bytes>> {
    instance_consumer(&e, &RegistryStorageKeys::TLDs, value)
}

pub fn nfd(e: &Env, value: Option<Address>) -> Option<Address> {
    instance_consumer(&e, &RegistryStorageKeys::NFD, value)
}

pub fn treasury(e: &Env, value: Option<Address>) -> Option<Address> {
    instance_consumer(&e, &RegistryStorageKeys::Treasury, value)
}

pub fn v1_records(e: &Env, registry: Option<Address>, max_snapshot: Option<u64>, deadline: Option<u64>) -> (Address, u64, u64) {
    (
        instance_consumer(&e, &RegistryStorageKeys::V1Registry, registry).unwrap(),
        instance_consumer(&e, &RegistryStorageKeys::V1MaxSnapshot, max_snapshot).unwrap(),
        instance_consumer(&e, &RegistryStorageKeys::V1Deadline, deadline).unwrap(),
    )
}

pub fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(LEDGER_WEEK, LEDGER_MONTH);
}

pub fn domain(e: &Env, node: &BytesN<32>, value: Option<Domain>) -> Option<Domain> {
    persistent_consumer(&e, &RegistryStorageKeys::Domain(node.clone()), value)
}

pub fn sub_domain(e: &Env, node: &BytesN<32>, value: Option<SubDomain>) -> Option<SubDomain> {
    persistent_consumer(&e, &RegistryStorageKeys::SubDomain(node.clone()), value)
}
