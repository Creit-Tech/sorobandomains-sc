use soroban_sdk::{contracttype, Address, BytesN, Env};

#[contracttype]
enum EventTopics {
    OfferAccepted,
}

pub fn emit_offer_accepted(
    e: &Env,
    buyer: &Address,
    seller: &Address,
    node: &BytesN<32>,
    amount: &u128,
) {
    e.events().publish(
        (EventTopics::OfferAccepted,),
        (
            buyer.clone(),
            seller.clone(),
            node.clone(),
            amount.clone(),
            e.ledger().timestamp(),
        ),
    );
}
