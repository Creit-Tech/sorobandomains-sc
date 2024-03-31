use soroban_sdk::{contracttype, Address, BytesN, Env};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Domain {
    // The `node` is the hash of the domain following the logic used by the function `generate_domain_node`
    pub node: BytesN<32>,

    // The owner of the node above and the address who can make updates
    pub owner: Address,

    // The address is where the node resolves to
    pub address: Address,

    // The TTL is the end expiration date of the domain.
    // A domain that have been expired for at least 30 days can be claimed by another address
    pub exp_date: u64,

    // The collateral is the amount of reserves the owner of the domain has deposited
    // For example, if the `node_rate` is 1 unit of collateral and the min ttl is a year then the collateral amount is:
    // 1 * (3600 * 24 * 365) = 3.1536000 XLM
    pub collateral: u128,

    // The snapshot is a value used as a flag for checking if other records are valid
    // The snapshot is the timestamp it was created
    pub snapshot: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct SubDomain {
    // The node is the hash of the subdomain
    pub node: BytesN<32>,

    // Parent is the hash of the root of the domain
    pub parent: BytesN<32>,

    // The address is where the node resolves to
    pub address: Address,

    // The snapshot is taken from the parent domain
    // If the subdomain snapshot is different from the parent one, it means the subdomain is invalid
    pub snapshot: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Record {
    Domain(Domain),
    SubDomain(SubDomain),
}

#[contracttype]
pub enum RecordKeys {
    Record(BytesN<32>),
    SubRecord(BytesN<32>),
}

pub trait RecordEntity {
    fn set_record(&self, record: &Record);
    fn record(&self, node: &RecordKeys) -> Option<Record>;
    fn bump_record(&self, record_type: &RecordKeys);
    fn burn_record(&self, record_type: &RecordKeys);
}

impl RecordEntity for Env {
    fn set_record(&self, record: &Record) {
        let key: RecordKeys = match record {
            Record::Domain(domain) => RecordKeys::Record(domain.node.clone()),
            Record::SubDomain(sub) => RecordKeys::SubRecord(sub.node.clone()),
        };
        self.storage().persistent().set(&key, record);
        self.bump_record(&key);
    }

    fn record(&self, node: &RecordKeys) -> Option<Record> {
        self.storage().persistent().get(node)
    }

    fn bump_record(&self, record_type: &RecordKeys) {
        self.storage().persistent().extend_ttl(
            record_type,
            17280,
            self.ledger().sequence() + (17280 * 30),
        );
    }

    fn burn_record(&self, record_type: &RecordKeys) {
        self.storage().persistent().remove(record_type)
    }
}
