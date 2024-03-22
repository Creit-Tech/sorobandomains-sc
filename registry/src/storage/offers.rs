use soroban_sdk::{contracttype, Address, BytesN, Env};

#[contracttype]
#[derive(Clone)]
pub struct BuyOffer {
    pub buyer: Address,

    // Domain node
    pub node: BytesN<32>,

    // Price for the sale, this price is set by the user.
    pub amount: u128,

    // The snapshot is taken from the domain being sold
    // Used to know if an offer is valid, or it got outdated
    pub snapshot: u64,
}

#[contracttype]
pub struct SaleOffer {
    // Domain node
    pub node: BytesN<32>,

    // Price for the sale, this price is set by the user.
    pub amount: u128,

    // The snapshot is taken from the domain being sold
    // Used to know if an offer is valid, or it got outdated
    pub snapshot: u64,
}

pub enum Offer {
    BuyOffer(BuyOffer),
    SaleOffer(SaleOffer),
}

#[contracttype]
pub enum OffersDataKeys {
    BuyOffer(BytesN<32>),
    SaleOffer(BytesN<32>),
}

pub struct Offers {
    env: Env,
}

impl Offers {
    #[inline(always)]
    pub(crate) fn new(env: &Env) -> Offers {
        Offers { env: env.clone() }
    }
    pub fn set(&self, offer: &Offer) {
        match offer {
            Offer::BuyOffer(value) => {
                self.env
                    .storage()
                    .persistent()
                    .set(&OffersDataKeys::BuyOffer(value.node.clone()), value);
            }
            Offer::SaleOffer(value) => {
                self.env
                    .storage()
                    .persistent()
                    .set(&OffersDataKeys::SaleOffer(value.node.clone()), value);
            }
        }
    }
    pub fn get(&self, key: &OffersDataKeys) -> Option<Offer> {
        match key {
            OffersDataKeys::BuyOffer(_) => {
                if let Some(record) = self.env.storage().persistent().get(key) {
                    Some(Offer::BuyOffer(record))
                } else {
                    None
                }
            }
            OffersDataKeys::SaleOffer(_) => {
                if let Some(record) = self.env.storage().persistent().get(key) {
                    Some(Offer::SaleOffer(record))
                } else {
                    None
                }
            }
        }
    }
    pub fn bump(&self, key: &OffersDataKeys) {
        self.env.storage().persistent().extend_ttl(
            key,
            17280,
            self.env.ledger().sequence() + (17280 * 30),
        )
    }
    pub fn burn(&self, key: &OffersDataKeys) {
        self.env.storage().persistent().remove(key);
    }
}

pub trait OffersFunc {
    fn _offers(&self) -> Offers;
}

impl OffersFunc for Env {
    fn _offers(&self) -> Offers {
        Offers::new(self)
    }
}
