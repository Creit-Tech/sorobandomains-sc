use crate::errors::RegistryV2Errors;
use crate::events::{ClaimRecord, DomainEvicted, RegistryDomain, RegistrySubDomain, RenewDomain, UpdateRecord};
use crate::non_fungible_domain::Client as NonFungibleDomainClient;
use crate::registry::{Client as RegistryContractClient, Record, RecordKeys};
use crate::storage::{
    admin, consume_index, domain, extend_instance, nfd, oracle, paying_asset, sub_domain, tlds, treasury, v1_records, Domain, RecordKey,
    SubDomain,
};
use crate::utils::{burn_token, calculate_exp_date, mint_token, pay_domain_time, validate_domain_expiration, validate_outdated_subdomain};
use common::utils::{generate_node, validate_domain};
use soroban_sdk::token::TokenClient;
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Bytes, BytesN, Env, Vec};

pub trait RegistryV2ContractTrait {
    fn __constructor(
        e: Env,
        new_admin: Address,
        new_oracle: Address,
        new_paying_asset: Address,
        new_tlds: Vec<Bytes>,
        new_nfd: Address,
        v1_registry: Address,
    );

    fn upgrade(e: Env, hash: BytesN<32>);

    fn update_treasury(e: Env, new_treasury: Address);

    fn withdraw(e: Env);

    fn register(e: Env, new_domain: Bytes, tld: Bytes, owner: Address, address: Address, periods: u64) -> Result<Domain, RegistryV2Errors>;

    fn register_sub(e: Env, parent: RecordKey, new_subdomain: Bytes, address: Address) -> Result<SubDomain, RegistryV2Errors>;

    fn update_address(e: Env, record_key: RecordKey, new_address: Address) -> Result<(), RegistryV2Errors>;

    fn record(e: Env, record_key: RecordKey) -> Result<(Domain, Option<SubDomain>), RegistryV2Errors>;

    fn parse_domain(e: Env, domain: Bytes, tld: Bytes) -> BytesN<32>;

    /// This method will increase the exp date from the domain
    /// Anyone can call this method and pay for the extension of a domain
    fn renew(e: Env, caller: Address, node: BytesN<32>, periods: u64) -> Result<Domain, RegistryV2Errors>;

    /// This method can be called to claim an expired domain
    /// The one claiming the domain will pay the fee for the new registration
    fn claim(e: Env, caller: Address, node: BytesN<32>, address: Address, periods: u64) -> Result<Domain, RegistryV2Errors>;

    /// This method is used by the admin to burn a domain
    fn evict(e: Env, node: BytesN<32>) -> Result<(), RegistryV2Errors>;

    /// This method is used to migrate domains from the v1
    /// The migration must be done
    fn migrate(e: Env, new_domain: Bytes, tld: Bytes) -> Result<Domain, RegistryV2Errors>;
}

#[contract]
pub struct RegistryV2Contract;

#[contractimpl]
impl RegistryV2ContractTrait for RegistryV2Contract {
    fn __constructor(
        e: Env,
        new_admin: Address,
        new_oracle: Address,
        new_paying_asset: Address,
        new_tlds: Vec<Bytes>,
        new_nfd: Address,
        v1_registry: Address,
    ) {
        admin(&e, Some(new_admin));
        oracle(&e, Some(new_oracle));
        paying_asset(&e, Some(new_paying_asset));
        tlds(&e, Some(new_tlds));
        nfd(&e, Some(new_nfd));
        v1_records(
            &e,
            Some(v1_registry),
            Some(e.ledger().timestamp()),
            Some(e.ledger().timestamp() + (3600 * 24 * 90)),
        );
        extend_instance(&e);
    }

    fn upgrade(e: Env, hash: BytesN<32>) {
        admin(&e, None).unwrap().require_auth();
        e.deployer().update_current_contract_wasm(hash);
        extend_instance(&e);
    }

    fn update_treasury(e: Env, new_treasury: Address) {
        admin(&e, None).unwrap().require_auth();
        treasury(&e, Some(new_treasury));
        extend_instance(&e);
    }

    fn withdraw(e: Env) {
        let token: TokenClient = TokenClient::new(&e, &paying_asset(&e, None).unwrap());
        let balance: i128 = token.balance(&e.current_contract_address());
        token.transfer(&e.current_contract_address(), &treasury(&e, None).unwrap(), &balance);
        extend_instance(&e);
    }

    fn register(e: Env, new_domain: Bytes, tld: Bytes, owner: Address, address: Address, periods: u64) -> Result<Domain, RegistryV2Errors> {
        owner.require_auth();

        if validate_domain(&new_domain).is_err() {
            return Err(RegistryV2Errors::InvalidDomain);
        }

        if !tlds(&e, None).unwrap().contains(tld.clone()) {
            return Err(RegistryV2Errors::UnsupportedTLD);
        }

        let node_hash: BytesN<32> = generate_node(&e, &new_domain, &tld);

        if domain(&e, &node_hash, None).is_some() {
            return Err(RegistryV2Errors::DomainAlreadyExist);
        }

        // During the migration period we check if the domain is part of the v1 registry and hasn't been migrated
        // If the domain snapshot is lower to the snapshot and the domain hasn't migrated to the v2 then panic
        let (registry, max_snapshot, deadline) = v1_records(&e, None, None, None);
        if e.ledger().timestamp() < deadline {
            let v1_result = RegistryContractClient::new(&e, &registry).try_record(&RecordKeys::Record(node_hash.clone()));

            match v1_result {
                Ok(result) => {
                    let record: Option<Record> = result.unwrap();
                    if let Some(v) = record {
                        if let Record::Domain(v1_domain) = v {
                            if v1_domain.snapshot <= max_snapshot {
                                panic_with_error!(&e, RegistryV2Errors::V1DomainRegistered);
                            }
                        }
                    }
                }
                Err(_) => {}
            };
        }

        let (_, xlm_amount) = pay_domain_time(&e, &owner, &new_domain)?;

        let new_domain_record: Domain = Domain {
            domain: new_domain,
            tld,
            node: node_hash,
            token_id: consume_index(&e),
            address,
            exp_date: calculate_exp_date(&e, &periods),
            snapshot: e.ledger().timestamp(),
        };

        mint_token(&e, &owner, &new_domain_record.token_id, &new_domain_record.node)?;

        RegistryDomain {
            register: owner,
            domain: new_domain_record.domain.clone(),
            tld: new_domain_record.tld.clone(),
            address: new_domain_record.address.clone(),
            exp_date: new_domain_record.exp_date.clone(),
            amount_paid: xlm_amount,
        }
        .publish(&e);

        extend_instance(&e);
        Ok(domain(&e, &new_domain_record.node, Some(new_domain_record.clone())).unwrap())
    }

    fn register_sub(e: Env, parent: RecordKey, new_subdomain: Bytes, address: Address) -> Result<SubDomain, RegistryV2Errors> {
        if validate_domain(&new_subdomain).is_err() {
            return Err(RegistryV2Errors::InvalidDomain);
        }

        let root: BytesN<32>;
        let snapshot: u64;
        let token_id: u32;
        let parent_node: BytesN<32> = match parent {
            RecordKey::Domain(id) => {
                let saved_record: Domain =
                    domain(&e, &id, None).unwrap_or_else(|| panic_with_error!(&e, RegistryV2Errors::RecordDoesntExist));
                validate_domain_expiration(&e, &saved_record)?;
                token_id = saved_record.token_id;
                root = saved_record.node;
                snapshot = saved_record.snapshot;
                id
            }
            RecordKey::SubDomain(id) => {
                let saved_subdomain: SubDomain =
                    sub_domain(&e, &id, None).unwrap_or_else(|| panic_with_error!(&e, RegistryV2Errors::RecordDoesntExist));
                let root_domain: Domain = domain(&e, &saved_subdomain.root, None).unwrap();
                validate_domain_expiration(&e, &root_domain)?;
                token_id = root_domain.token_id;
                root = root_domain.node;
                snapshot = root_domain.snapshot;
                id
            }
        };

        let node_hash: BytesN<32> = generate_node(&e, &new_subdomain, &(Bytes::from(parent_node.clone())));
        if sub_domain(&e, &node_hash, None).is_some() {
            return Err(RegistryV2Errors::DomainAlreadyExist);
        }

        NonFungibleDomainClient::new(&e, &nfd(&e, None).unwrap())
            .owner_of(&token_id)
            .require_auth();

        let new_subdomain_record: SubDomain = SubDomain {
            domain: new_subdomain,
            node: node_hash.clone(),
            parent: parent_node,
            address,
            root,
            snapshot,
        };

        RegistrySubDomain {
            domain: new_subdomain_record.domain.clone(),
            parent: new_subdomain_record.parent.clone(),
            address: new_subdomain_record.address.clone(),
        }
        .publish(&e);

        sub_domain(&e, &node_hash, Some(new_subdomain_record.clone()));
        extend_instance(&e);
        Ok(new_subdomain_record)
    }

    fn update_address(e: Env, record_key: RecordKey, new_address: Address) -> Result<(), RegistryV2Errors> {
        let token_id: u32;
        let old_address: Address;
        let node: BytesN<32> = match record_key.clone() {
            RecordKey::Domain(id) => {
                let mut saved_record: Domain =
                    domain(&e, &id, None).unwrap_or_else(|| panic_with_error!(&e, RegistryV2Errors::RecordDoesntExist));
                validate_domain_expiration(&e, &saved_record)?;
                token_id = saved_record.token_id;
                old_address = saved_record.address;
                saved_record.address = new_address.clone();
                domain(&e, &id, Some(saved_record));
                id
            }
            RecordKey::SubDomain(id) => {
                let mut saved_record: SubDomain =
                    sub_domain(&e, &id, None).unwrap_or_else(|| panic_with_error!(&e, RegistryV2Errors::RecordDoesntExist));
                let saved_domain: Domain = domain(&e, &saved_record.root, None).unwrap();
                validate_domain_expiration(&e, &saved_domain)?;
                token_id = saved_domain.token_id;
                old_address = saved_record.address;
                saved_record.address = new_address.clone();
                sub_domain(&e, &id, Some(saved_record));
                id
            }
        };

        NonFungibleDomainClient::new(&e, &nfd(&e, None).unwrap())
            .owner_of(&token_id)
            .require_auth();

        UpdateRecord {
            node,
            from: old_address,
            to: new_address,
        }
        .publish(&e);

        extend_instance(&e);
        Ok(())
    }

    fn record(e: Env, record_key: RecordKey) -> Result<(Domain, Option<SubDomain>), RegistryV2Errors> {
        let (saved_domain, saved_subdomain) = match record_key {
            RecordKey::Domain(id) => (
                domain(&e, &id, None).unwrap_or_else(|| panic_with_error!(&e, RegistryV2Errors::RecordDoesntExist)),
                None,
            ),
            RecordKey::SubDomain(id) => {
                let saved_record: SubDomain =
                    sub_domain(&e, &id, None).unwrap_or_else(|| panic_with_error!(&e, RegistryV2Errors::RecordDoesntExist));
                let saved_domain: Domain = domain(&e, &saved_record.root, None).unwrap();
                (saved_domain, Some(saved_record))
            }
        };

        validate_domain_expiration(&e, &saved_domain)?;
        if let Some(v) = saved_subdomain.clone() {
            validate_outdated_subdomain(&saved_domain, &v)?;
        }
        extend_instance(&e);

        Ok((saved_domain, saved_subdomain))
    }

    fn parse_domain(e: Env, domain: Bytes, tld: Bytes) -> BytesN<32> {
        extend_instance(&e);
        generate_node(&e, &domain, &tld)
    }

    fn renew(e: Env, caller: Address, node: BytesN<32>, periods: u64) -> Result<Domain, RegistryV2Errors> {
        caller.require_auth();
        let mut saved_record: Domain =
            domain(&e, &node, None).unwrap_or_else(|| panic_with_error!(&e, RegistryV2Errors::RecordDoesntExist));

        let (_, xlm_amount) = pay_domain_time(&e, &caller, &saved_record.domain)?;

        saved_record.exp_date = calculate_exp_date(&e, &periods);

        RenewDomain {
            node: node.clone(),
            payer: caller,
            amount_paid: xlm_amount,
            exp_date: saved_record.exp_date.clone(),
        }
        .publish(&e);

        extend_instance(&e);
        Ok(domain(&e, &node, Some(saved_record)).unwrap())
    }

    fn claim(e: Env, caller: Address, node: BytesN<32>, address: Address, periods: u64) -> Result<Domain, RegistryV2Errors> {
        caller.require_auth();
        let mut saved_record: Domain =
            domain(&e, &node, None).unwrap_or_else(|| panic_with_error!(&e, RegistryV2Errors::RecordDoesntExist));

        if e.ledger().timestamp() < saved_record.exp_date + (3600 * 24 * 30) {
            return Err(RegistryV2Errors::RecordCantBeClaimedYet);
        }

        let (_, xlm_amount) = pay_domain_time(&e, &caller, &saved_record.domain)?;

        burn_token(&e, &saved_record.token_id)?;
        mint_token(&e, &caller, &saved_record.token_id, &saved_record.node)?;

        saved_record.exp_date = calculate_exp_date(&e, &periods);
        saved_record.snapshot = e.ledger().timestamp();
        saved_record.address = address.clone();

        ClaimRecord {
            node: saved_record.node.clone(),
            register: caller.clone(),
            address: saved_record.address.clone(),
            exp_date: saved_record.exp_date.clone(),
            amount_paid: xlm_amount,
        }
        .publish(&e);

        extend_instance(&e);
        Ok(domain(&e, &node, Some(saved_record)).unwrap())
    }

    fn evict(e: Env, node: BytesN<32>) -> Result<(), RegistryV2Errors> {
        admin(&e, None).unwrap().require_auth();
        let saved_record: Domain = domain(&e, &node, None).unwrap_or_else(|| panic_with_error!(&e, RegistryV2Errors::RecordDoesntExist));
        burn_token(&e, &saved_record.token_id)?;
        DomainEvicted { node }.publish(&e);
        extend_instance(&e);
        Ok(())
    }

    fn migrate(e: Env, new_domain: Bytes, tld: Bytes) -> Result<Domain, RegistryV2Errors> {
        let node: BytesN<32> = generate_node(&e, &new_domain, &tld);

        if domain(&e, &node, None).is_some() {
            panic_with_error!(&e, RegistryV2Errors::DomainAlreadyExist);
        }

        let (registry, max_snapshot, deadline) = v1_records(&e, None, None, None);

        if e.ledger().timestamp() > deadline {
            panic_with_error!(&e, RegistryV2Errors::V1DomainMigrationExpired);
        }

        let v1_record = RegistryContractClient::new(&e, &registry)
            .record(&RecordKeys::Record(node.clone()))
            .unwrap_or_else(|| panic_with_error!(&e, RegistryV2Errors::RecordDoesntExist));

        let v1_domain = match v1_record {
            Record::Domain(v) => v,
            Record::SubDomain(_) => {
                panic_with_error!(&e, RegistryV2Errors::InvalidV1Domain);
            }
        };

        if v1_domain.snapshot > max_snapshot {
            panic_with_error!(&e, RegistryV2Errors::InvalidV1Domain);
        }

        v1_domain.owner.require_auth();

        let new_domain_record: Domain = Domain {
            domain: new_domain,
            tld,
            node: v1_domain.node.clone(),
            token_id: consume_index(&e),
            address: v1_domain.address.clone(),
            exp_date: calculate_exp_date(&e, &1),
            snapshot: e.ledger().timestamp(),
        };

        mint_token(&e, &v1_domain.owner, &new_domain_record.token_id, &new_domain_record.node)?;

        RegistryDomain {
            register: v1_domain.owner,
            domain: new_domain_record.domain.clone(),
            tld: new_domain_record.tld.clone(),
            address: new_domain_record.address.clone(),
            exp_date: new_domain_record.exp_date.clone(),
            amount_paid: 0,
        }
        .publish(&e);

        extend_instance(&e);
        Ok(domain(&e, &new_domain_record.node, Some(new_domain_record.clone())).unwrap())
    }
}
