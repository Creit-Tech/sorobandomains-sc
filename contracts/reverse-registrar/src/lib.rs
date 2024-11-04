#![no_std]

mod contract;
mod errors;
mod events;
mod tests;
mod types;

mod registry {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/registry.wasm"
    );
}
