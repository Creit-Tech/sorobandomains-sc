use crate::errors::ContractErrors;
use crate::storage::core::CoreData;
use crate::storage::offers::{BuyOffer, Offer, OffersDataKeys, OffersFunc, SaleOffer};
use crate::storage::record::Domain;
use soroban_sdk::{panic_with_error, token, Address, Env};

// Sets a domain SaleOffer
// Sales offers don't require a collateral from the seller
// They don't have any type of requirement beside the amount being higher than the domain collateral
pub fn set_sale_offer(e: &Env, domain: &Domain, amount: &u128) {
    let new_sale_offer: SaleOffer = SaleOffer {
        node: domain.node.clone(),
        amount: amount.clone(),
        snapshot: domain.snapshot.clone(),
    };

    e._offers().set(&Offer::SaleOffer(new_sale_offer));
    e._offers()
        .bump(&OffersDataKeys::SaleOffer(domain.node.clone()));
}

// Sets a new BuyOffer
// The buyer needs to put the same amount of collateral for that domain
// The buyer needs to put the amount they are offering to the seller
pub fn set_new_buy_offer(
    e: &Env,
    core_data: &CoreData,
    caller: &Address,
    domain: &Domain,
    amount: &u128,
) {
    let new_offer: BuyOffer = BuyOffer {
        buyer: caller.clone(),
        node: domain.node.clone(),
        amount: amount.clone(),
        snapshot: domain.snapshot.clone(),
    };

    token::Client::new(&e, &core_data.col_asset).transfer(
        &caller,
        &e.current_contract_address(),
        &(amount.clone() as i128),
    );

    e._offers().set(&Offer::BuyOffer(new_offer));
    e._offers()
        .bump(&OffersDataKeys::BuyOffer(domain.node.clone()));
}

// Updates an already existing BuyOffer
// It checks if the new amount is lower than the old offer
// If it's the same caller, it only updates the values and take the extra collateral
// If it's a new caller, we release the old collateral, take the new collateral and update the offer
pub fn update_buy_offer(
    e: &Env,
    core_data: &CoreData,
    caller: &Address,
    old_buy_offer: &BuyOffer,
    domain: &Domain,
    amount: &u128,
) {
    if amount <= &old_buy_offer.amount {
        panic_with_error!(&e, &ContractErrors::InvalidOfferAmount);
    }

    let mut updated_offer: BuyOffer;

    if caller == &old_buy_offer.buyer {
        let extra_amt: u128 = amount - old_buy_offer.amount;
        updated_offer = old_buy_offer.clone();
        updated_offer.amount += extra_amt;

        token::Client::new(&e, &core_data.col_asset).transfer(
            &caller,
            &e.current_contract_address(),
            &(extra_amt as i128),
        );
    } else {
        updated_offer = BuyOffer {
            buyer: caller.clone(),
            node: domain.node.clone(),
            amount: amount.clone(),
            snapshot: domain.snapshot.clone(),
        };

        token::Client::new(&e, &core_data.col_asset).transfer(
            &e.current_contract_address(),
            &old_buy_offer.buyer,
            &(old_buy_offer.amount as i128),
        );

        token::Client::new(&e, &core_data.col_asset).transfer(
            &caller,
            &e.current_contract_address(),
            &(amount.clone() as i128),
        );
    }

    e._offers().set(&Offer::BuyOffer(updated_offer));
    e._offers()
        .bump(&OffersDataKeys::BuyOffer(domain.node.clone()));
}
