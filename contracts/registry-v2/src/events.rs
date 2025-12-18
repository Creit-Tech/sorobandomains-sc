use soroban_sdk::{contractevent, Address, Bytes, BytesN};

#[contractevent(topics = ["REGISTRY", "DOMAIN"])]
pub struct RegistryDomain {
    pub register: Address,
    pub domain: Bytes,
    pub tld: Bytes,
    pub address: Address,
    pub exp_date: u64,
    pub amount_paid: u128,
}

#[contractevent(topics = ["REGISTRY", "SUBDOMAIN"])]
pub struct RegistrySubDomain {
    pub domain: Bytes,
    pub parent: BytesN<32>,
    pub address: Address,
}

#[contractevent(topics = ["UPDATE", "RECORD"])]
pub struct UpdateRecord {
    #[topic]
    pub node: BytesN<32>,
    pub from: Address,
    pub to: Address,
}

#[contractevent(topics = ["RENEW", "DOMAIN"])]
pub struct RenewDomain {
    #[topic]
    pub node: BytesN<32>,
    pub payer: Address,
    pub amount_paid: u128,
    pub exp_date: u64,
}

#[contractevent(topics = ["CLAIM", "DOMAIN"])]
pub struct ClaimRecord {
    #[topic]
    pub node: BytesN<32>,
    pub register: Address,
    pub address: Address,
    pub exp_date: u64,
    pub amount_paid: u128,
}

#[contractevent(topics = ["EVICT", "DOMAIN"])]
pub struct DomainEvicted {
    #[topic]
    pub node: BytesN<32>,
}
