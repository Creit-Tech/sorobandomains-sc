use soroban_sdk::{contracttype, Bytes, BytesN, String, Symbol};

#[contracttype]
pub enum StorageKeys {
    // Instance keys
    Admin,
    Registry,
    Fee,
    Currency,
    Treasury,

    // Persistent keys
    Key((BytesN<32>, Symbol)), // -> Returns (Value, u64) where the u64 value is the domain snapshot
}

#[contracttype]
#[derive(Clone)]
pub enum Value {
    String(String),
    Bytes(Bytes),
    Number(i128),
}
