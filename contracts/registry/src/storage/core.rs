use soroban_sdk::{contracttype, Address, Bytes, Env, Vec};

#[contracttype]
pub struct CoreData {
    // Admin can upgrade the contract and liquidate nodes
    pub adm: Address,

    // @deprecated - We don't use this anymore
    // The node rate is the amount of collateral the creator needs to put as collateral in terms of seconds
    // For example a node rate can be as low as 1 unit of collateral (ex: 0.0000001 XLM)
    pub node_rate: u128,

    // Address of the asset used as collateral (XLM)
    pub col_asset: Address,

    // The min amount of time a domain can be registered
    pub min_duration: u64,

    pub allowed_tlds: Vec<Bytes>,
}

#[contracttype]
pub struct OffersConfig {
    pub fee_taker: Address,
    pub fee: u128,
}

#[contracttype]
pub enum CoreDataKeys {
    CoreData,
    OffersConfig,
    Oracle,
}

pub trait CoreDataEntity {
    fn bump_core(&self);
    fn set_core_data(&self, core_data: &CoreData);
    fn core_data(&self) -> Option<CoreData>;
    fn set_oracle(&self, oracle: &Address);
    fn oracle(&self) -> Address;
    fn set_offers_config(&self, v: &OffersConfig);
    fn offers_config(&self) -> Option<OffersConfig>;
    fn is_adm(&self);
}

impl CoreDataEntity for Env {
    fn bump_core(&self) {
        self.storage().instance().extend_ttl(17280, 17280 * 30);
    }

    fn set_core_data(&self, core_data: &CoreData) {
        self.storage()
            .instance()
            .set(&CoreDataKeys::CoreData, core_data);
        self.bump_core();
    }

    fn core_data(&self) -> Option<CoreData> {
        self.storage().instance().get(&CoreDataKeys::CoreData)
    }

    fn set_oracle(&self, oracle: &Address) {
        self.storage().instance().set(&CoreDataKeys::Oracle, oracle);
    }

    fn oracle(&self) -> Address {
        self.storage()
            .instance()
            .get(&CoreDataKeys::Oracle)
            .unwrap()
    }

    fn set_offers_config(&self, v: &OffersConfig) {
        self.storage()
            .instance()
            .set(&CoreDataKeys::OffersConfig, v);
    }

    fn offers_config(&self) -> Option<OffersConfig> {
        self.storage().instance().get(&CoreDataKeys::OffersConfig)
    }

    fn is_adm(&self) {
        self.core_data().unwrap().adm.require_auth();
    }
}
