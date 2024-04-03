#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

#[contracttype]
pub enum CoreDataKeys {
    Adm,
}

pub trait TreasuryContractTrait {
    fn init(e: Env, adm: Address);
    fn upgrade(e: Env, hash: BytesN<32>);
}

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContractTrait for TreasuryContract {
    fn init(e: Env, adm: Address) {
        if e.storage()
            .instance()
            .get::<CoreDataKeys, Address>(&CoreDataKeys::Adm)
            .is_some()
        {
            panic!();
        }

        e.storage().instance().set(&CoreDataKeys::Adm, &adm);
    }

    fn upgrade(e: Env, hash: BytesN<32>) {
        e.storage()
            .instance()
            .get::<CoreDataKeys, Address>(&CoreDataKeys::Adm)
            .unwrap()
            .require_auth();

        e.deployer().update_current_contract_wasm(hash);
    }
}
