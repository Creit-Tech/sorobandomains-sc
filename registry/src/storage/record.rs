use soroban_sdk::{contracttype, Address, BytesN, Env};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    /// The `node` is the hash of the domain following the logic used by the function `generate_domain_node`
    pub node: BytesN<32>,

    /// The owner of the node above and the address who can make updates
    pub owner: Address,

    /// The address is where the node resolves to
    pub address: Address,

    /// The TTL is the end expiration date of the domain.
    /// A domain that have been expired for at least 30 days can be claimed by another address
    pub exp_date: u64,

    /// The collateral is the amount of reserves the owner of the domain has deposited
    /// For example, if the `node_rate` is 1 unit of collateral and the min ttl is a year then the collateral amount is:
    /// 1 * (3600 * 24 * 365) = 3.1536000 XLM
    pub collateral: u128,
}

#[contracttype]
pub struct SubRecord {
    /// The node is the hash of the subdomain
    pub node: BytesN<32>,

    /// Parent is the hash of the root of the domain
    pub parent: BytesN<32>,

    /// The address is where the node resolves to
    pub address: Address,
}

#[contracttype]
pub enum RecordKeys {
    Record(BytesN<32>),
    SubRecord(BytesN<32>),
}

pub trait RecordEntity {
    fn set_record(&self, record: &Record);
    fn set_sub(&self, record: &SubRecord);
    fn record(&self, node: &BytesN<32>) -> Option<Record>;
    fn bump_record(&self, node: &BytesN<32>);
    fn bump_sub(&self, node: &BytesN<32>);
    fn burn_record(&self, node: &BytesN<32>);
}

impl RecordEntity for Env {
    fn set_record(&self, record: &Record) {
        self.storage()
            .persistent()
            .set(&RecordKeys::Record(record.node.clone()), record);
        self.bump_record(&record.node);
    }

    fn set_sub(&self, record: &SubRecord) {
        self.storage()
            .persistent()
            .set(&RecordKeys::SubRecord(record.node.clone()), record);
        self.bump_sub(&record.node);
    }

    fn record(&self, node: &BytesN<32>) -> Option<Record> {
        self.storage()
            .persistent()
            .get(&RecordKeys::Record(node.clone()))
    }

    fn bump_record(&self, node: &BytesN<32>) {
        self.storage().persistent().extend_ttl(
            &RecordKeys::Record(node.clone()),
            17280,
            self.ledger().sequence() + (17280 * 30),
        );
    }

    fn bump_sub(&self, node: &BytesN<32>) {
        self.storage().persistent().extend_ttl(
            &RecordKeys::SubRecord(node.clone()),
            17280,
            self.ledger().sequence() + (17280 * 30),
        );
    }

    fn burn_record(&self, node: &BytesN<32>) {
        self.storage()
            .persistent()
            .remove(&RecordKeys::Record(node.clone()))
    }
}
