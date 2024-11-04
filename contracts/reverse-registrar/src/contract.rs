use crate::{
    errors::Error,
    events::emit_domain_updated,
    registry::{ContractErrors as RegistryErrors, Record, RecordKeys},
    types::{CoreDataKeys, Domain},
};
use common::utils::generate_node;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, Address, BytesN, Env, IntoVal,
};

const LEDGER_DAY: u32 = 17_280;

pub trait ReverseRegistrarTrait {
    /// Initialize the contract with the admin and registry addresses.
    ///
    /// # Arguments
    ///
    /// * `e` - The environment in which the contract is executed.
    /// * `admin` - The address of the admin.
    /// * `registry` - The address of the registry.
    ///
    /// # Returns
    ///
    /// * `Result<(), Error>` - Returns an error if the contract is already initialized.
    fn init(e: Env, admin: Address, registry: Address) -> Result<(), Error>;

    /// Upgrade the contract to a new version.
    ///
    /// # Arguments
    ///
    /// * `e` - The environment in which the contract is executed.
    /// * `hash` - The hash of the new contract.
    fn upgrade(e: Env, hash: BytesN<32>);

    /// Set the reverse domain for an address.
    ///
    /// # Arguments
    ///
    /// * `e` - The environment in which the contract is executed.
    /// * `address` - The address for which the reverse domain is being set.
    /// * `domain` - The reverse domain to set, or `None` to remove the reverse domain.
    ///
    /// # Returns
    ///
    /// * `Result<(), Error>` - Returns an error if the operation fails.
    fn set(e: Env, address: Address, domain: Option<Domain>) -> Result<(), Error>;

    /// Get the reverse domain for an address.
    ///
    /// # Arguments
    ///
    /// * `e` - The environment in which the contract is executed.
    /// * `address` - The address for which the reverse domain is being retrieved.
    ///
    /// # Returns
    ///
    /// * `Option<Domain>` - The reverse domain if it exists, otherwise `None`.
    fn get(e: Env, address: Address) -> Option<Domain>;
}

#[contract]
pub struct ReverseRegistrar;

#[contractimpl]
impl ReverseRegistrarTrait for ReverseRegistrar {
    fn init(e: Env, admin: Address, registry: Address) -> Result<(), Error> {
        if e.storage().instance().has(&CoreDataKeys::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        e.storage().instance().set(&CoreDataKeys::Admin, &admin);
        e.storage()
            .instance()
            .set(&CoreDataKeys::Registry, &registry);
        Ok(())
    }

    fn upgrade(e: Env, hash: BytesN<32>) {
        e.storage()
            .instance()
            .get::<CoreDataKeys, Address>(&CoreDataKeys::Admin)
            .unwrap()
            .require_auth();

        e.deployer().update_current_contract_wasm(hash);
    }

    fn set(e: Env, address: Address, domain: Option<Domain>) -> Result<(), Error> {
        address.require_auth();
        bump_instance(&e);

        if let Some(domain) = domain {
            validate_reverse_record(&e, &address, &domain)?;
            e.storage().persistent().set(&address, &domain);
            emit_domain_updated(&e, address, Some::<Domain>(domain));
        } else {
            e.storage().persistent().remove(&address);
            emit_domain_updated(&e, address, None::<Domain>);
        }
        Ok(())
    }

    fn get(e: Env, address: Address) -> Option<Domain> {
        bump_instance(&e);
        if let Some(domain) = e.storage().persistent().get(&address) {
            if validate_reverse_record(&e, &address, &domain).is_ok() {
                bump_record(&e, &address);
                return Some(domain);
            }
        }
        None
    }
}

fn validate_reverse_record(e: &Env, address: &Address, domain: &Domain) -> Result<(), Error> {
    let mut domain_node = generate_node(&e, &domain.sld, &domain.tld);
    if domain.subdomains.is_empty() {
        let Record::Domain(sld_domain) = fetch_domain_record(&e, &domain_node, false)? else {
            panic!("unreachable");
        };
        return match sld_domain.address == *address {
            true => Ok(()),
            false => Err(Error::AddressMismatch),
        };
    }

    if domain.subdomains.len() != 1 {
        // Only one subdomain is allowed.
        // If registry allows more than one subdomain, this contract should be updated.
        return Err(Error::NotImplemented);
    }

    domain_node = generate_node(
        &e,
        &domain.subdomains.first().unwrap(),
        &domain_node.try_into().unwrap(),
    );
    let Record::SubDomain(subdomain) = fetch_domain_record(&e, &domain_node, true)? else {
        panic!("unreachable");
    };

    match subdomain.address == *address {
        true => Ok(()),
        false => Err(Error::AddressMismatch),
    }
    // The expiration time and snapshot have already been checked in registry.
}

fn fetch_domain_record(e: &Env, node: &BytesN<32>, sub_record: bool) -> Result<Record, Error> {
    let registry = e.storage().instance().get(&CoreDataKeys::Registry).unwrap();
    let key = if sub_record {
        RecordKeys::SubRecord(node.clone())
    } else {
        RecordKeys::Record(node.clone())
    };

    let result = e.try_invoke_contract::<Option<Record>, RegistryErrors>(
        &registry,
        &symbol_short!("record"),
        (key,).into_val(e),
    );

    if result.is_err() {
        return Err(Error::FailedToGetRecord);
    }

    let record: Record = result
        .unwrap()
        .unwrap() // If conversion fails, it's a bug, so we panic.
        .unwrap_or_else(|| panic_with_error!(&e, &Error::FailedToGetRecord));

    Ok(record)
}

fn bump_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_DAY * 30, LEDGER_DAY * 60);
}

fn bump_record(e: &Env, address: &Address) {
    e.storage()
        .persistent()
        .extend_ttl(address, LEDGER_DAY * 30, LEDGER_DAY * 60);
}
