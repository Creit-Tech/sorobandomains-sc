use soroban_sdk::{contracttype, Address, Env};

use crate::types::Domain;

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
enum EventTopics {
    DomainUpdated = 1,
}

pub fn emit_domain_updated(e: &Env, address: Address, domain: Option<Domain>) {
    e.events()
        .publish((EventTopics::DomainUpdated,), (address, domain));
}
