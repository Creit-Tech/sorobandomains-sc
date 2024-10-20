#![no_std]

const LEDGER_DAY: u32 = 17280;
mod contract;
mod errors;
mod tests;
mod types;
mod utils;

mod registry {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/registry.wasm"
    );
}
