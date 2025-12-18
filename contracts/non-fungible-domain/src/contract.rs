use crate::errors::NonFungibleTokenError;
use crate::events::{emit_approve, emit_approve_for_all, emit_burn, emit_mint, emit_transfer};
use crate::storage::{
    admin, approval, approval_for_all, balance, extend_instance, metadata, owner, registry, remove_approval, remove_approval_for_all,
    token_node, total_supply, ApprovalData, Metadata,
};
use crate::utils::transfer;
use soroban_sdk::{contract, contractimpl, panic_with_error, symbol_short, Address, BytesN, Env, IntoVal, String, Symbol, Val};

pub trait NonFungibleDomainTrait {
    fn __constructor(e: Env, admin: Address, new_registry: Address);
    fn upgrade(e: Env, hash: BytesN<32>);
    fn total_supply(e: Env) -> u32;
    fn mint(e: Env, to: Address, token_id: u32, node: BytesN<32>);
    fn burn(e: Env, token_id: u32);

    // ---- SEP-0050 methods
    fn balance(e: Env, owner: Address) -> u32;
    fn owner_of(e: Env, token_id: u32) -> Address;
    fn transfer(e: Env, from: Address, to: Address, token_id: u32);
    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, token_id: u32);
    fn approve(e: Env, approver: Address, approved: Address, token_id: u32, live_until_ledger: u32);
    fn approve_for_all(e: Env, owner: Address, operator: Address, live_until_ledger: u32);
    fn get_approved(e: Env, token_id: u32) -> Option<Address>;
    fn is_approved_for_all(e: Env, owner: Address, operator: Address) -> bool;
    fn name(e: Env) -> String;
    fn symbol(e: Env) -> String;
}

#[contract]
pub struct NonFungibleDomain;

#[contractimpl]
impl NonFungibleDomainTrait for NonFungibleDomain {
    fn __constructor(e: Env, new_admin: Address, new_registry: Address) {
        let base_uri: String = String::from_str(&e, "https://stellar.sorobandomains.org/tokens");
        let name: String = String::from_str(&e, "Non-Fungible Domains");
        let symbol: String = String::from_str(&e, "NFD");
        metadata(&e, Some(Metadata { name, symbol, base_uri }));
        admin(&e, Some(new_admin));
        registry(&e, Some(new_registry));
    }

    fn upgrade(e: Env, hash: BytesN<32>) {
        admin(&e, None).unwrap().require_auth();
        e.deployer().update_current_contract_wasm(hash);
        extend_instance(&e);
    }

    fn total_supply(e: Env) -> u32 {
        extend_instance(&e);
        total_supply(&e, None).unwrap_or(0)
    }

    fn mint(e: Env, to: Address, token_id: u32, node: BytesN<32>) {
        registry(&e, None).unwrap().require_auth();
        if owner(&e, &token_id, None).is_some() {
            panic_with_error!(&e, &NonFungibleTokenError::TokenIdAlreadyExist);
        }
        transfer(&e, None, Some(to.clone()), &token_id);
        token_node(&e, &token_id, Some(node));
        emit_mint(&e, &to, &token_id);
        extend_instance(&e);
    }

    fn burn(e: Env, token_id: u32) {
        registry(&e, None).unwrap().require_auth();
        let current_owner: Address =
            owner(&e, &token_id, None).unwrap_or_else(|| panic_with_error!(&e, NonFungibleTokenError::NonExistentToken));
        transfer(&e, Some(current_owner.clone()), None, &token_id);
        emit_burn(&e, &current_owner, &token_id);
        extend_instance(&e);
    }

    fn balance(e: Env, owner: Address) -> u32 {
        extend_instance(&e);
        balance(&e, &owner, None).unwrap_or(0u32)
    }

    fn owner_of(e: Env, token_id: u32) -> Address {
        extend_instance(&e);
        owner(&e, &token_id, None).unwrap_or_else(|| panic_with_error!(&e, NonFungibleTokenError::NonExistentToken))
    }

    fn transfer(e: Env, from: Address, to: Address, token_id: u32) {
        from.require_auth();
        let current_owner: Address =
            owner(&e, &token_id, None).unwrap_or_else(|| panic_with_error!(&e, NonFungibleTokenError::NonExistentToken));
        if current_owner != from {
            panic_with_error!(e, NonFungibleTokenError::IncorrectOwner);
        }

        // We invoke the registry record so we confirm the domain is not expired
        let key: (Symbol, BytesN<32>) = (symbol_short!("Domain"), token_node(&e, &token_id, None).unwrap());
        e.invoke_contract::<Val>(&registry(&e, None).unwrap(), &symbol_short!("record"), (key,).into_val(&e.clone()));

        transfer(&e, Some(from.clone()), Some(to.clone()), &token_id);
        emit_transfer(&e, &from, &to, &token_id);
        extend_instance(&e);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, token_id: u32) {
        spender.require_auth();

        // If `spender` is not the owner, they must have explicit approval.
        let is_spender_owner: bool = spender == from;
        let is_spender_approved: bool = Self::get_approved(e.clone(), token_id.clone()) == Some(spender.clone());
        let has_spender_approval_for_all: bool = Self::is_approved_for_all(e.clone(), from.clone(), spender.clone());

        if !is_spender_owner && !is_spender_approved && !has_spender_approval_for_all {
            panic_with_error!(e, NonFungibleTokenError::InsufficientApproval);
        }

        // We invoke the registry record so we confirm the domain is not expired
        let key: (Symbol, BytesN<32>) = (symbol_short!("Domain"), token_node(&e, &token_id, None).unwrap());
        e.invoke_contract::<Val>(&registry(&e, None).unwrap(), &symbol_short!("record"), (key,).into_val(&e.clone()));

        transfer(&e, Some(from.clone()), Some(to.clone()), &token_id);
        emit_transfer(&e, &from, &to, &token_id);
        extend_instance(&e);
    }

    fn approve(e: Env, approver: Address, approved: Address, token_id: u32, live_until_ledger: u32) {
        approver.require_auth();

        let current_owner: Address =
            owner(&e, &token_id, None).unwrap_or_else(|| panic_with_error!(&e, NonFungibleTokenError::NonExistentToken));

        if current_owner != approver {
            panic_with_error!(e, NonFungibleTokenError::InvalidApprover);
        }

        if live_until_ledger == 0 {
            remove_approval(&e, &token_id);
        } else {
            if live_until_ledger < e.ledger().sequence() {
                panic_with_error!(e, NonFungibleTokenError::InvalidLiveUntilLedger);
            }

            approval(
                &e,
                &token_id,
                Some(ApprovalData {
                    approved: approved.clone(),
                    live_until_ledger: live_until_ledger.clone(),
                }),
            );
        }

        emit_approve(&e, &approver, &approved, &token_id, &live_until_ledger);
        extend_instance(&e);
    }

    fn approve_for_all(e: Env, owner: Address, operator: Address, live_until_ledger: u32) {
        owner.require_auth();

        if live_until_ledger == 0 {
            remove_approval_for_all(&e, &owner, &operator);
        } else {
            if live_until_ledger < e.ledger().sequence() {
                panic_with_error!(e, NonFungibleTokenError::InvalidLiveUntilLedger);
            }
            approval_for_all(&e, &owner, &operator, Some(live_until_ledger.clone()));
        }

        emit_approve_for_all(&e, &owner, &operator, &live_until_ledger);
        extend_instance(&e);
    }

    fn get_approved(e: Env, token_id: u32) -> Option<Address> {
        match approval(&e, &token_id, None) {
            None => None,
            Some(v) => {
                if v.live_until_ledger < e.ledger().sequence() {
                    None
                } else {
                    Some(v.approved)
                }
            }
        }
    }

    fn is_approved_for_all(e: Env, owner: Address, operator: Address) -> bool {
        match approval_for_all(&e, &owner, &operator, None) {
            None => false,
            Some(v) => v >= e.ledger().sequence(),
        }
    }

    fn name(e: Env) -> String {
        metadata(&e, None).unwrap().name
    }

    fn symbol(e: Env) -> String {
        metadata(&e, None).unwrap().symbol
    }
}
