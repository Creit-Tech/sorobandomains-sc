#![no_std]

pub mod contract;
pub mod errors;
pub mod events;
pub mod storage;
pub mod utils;

pub mod non_fungible_domain {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../target/wasm32v1-none/release/non_fungible_domain.wasm");
}

pub mod registry {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../target/wasm32v1-none/release/registry.wasm");
}
