use crate::errors::RegistryV2Errors;
use crate::non_fungible_domain::Client as NonFungibleDomainClient;
use crate::storage::{nfd, oracle, paying_asset, Domain, SubDomain};
use common::oracle::record_price;
use soroban_sdk::{token, Address, Bytes, BytesN, Env};

pub fn validate_domain_expiration(e: &Env, domain: &Domain) -> Result<(), RegistryV2Errors> {
    if domain.exp_date < e.ledger().timestamp() {
        return Err(RegistryV2Errors::RecordIsExpired);
    }
    Ok(())
}

pub fn validate_outdated_subdomain(domain: &Domain, subdomain: &SubDomain) -> Result<(), RegistryV2Errors> {
    if domain.snapshot != subdomain.snapshot {
        return Err(RegistryV2Errors::RecordIsExpired);
    }
    Ok(())
}

pub fn pay_domain_time(e: &Env, payer: &Address, domain: &Bytes) -> Result<(u128, u128), RegistryV2Errors> {
    let (usd_amount, xlm_amount) = record_price(&e, &oracle(&e, None).unwrap(), domain.len());
    let payment_result =
        token::Client::new(&e, &paying_asset(&e, None).unwrap()).try_transfer(&payer, &e.current_contract_address(), &(xlm_amount as i128));
    if payment_result.is_err() {
        return Err(RegistryV2Errors::PaymentFailed);
    }
    Ok((usd_amount, xlm_amount))
}

pub fn mint_token(e: &Env, to: &Address, token_id: &u32, node: &BytesN<32>) -> Result<(), RegistryV2Errors> {
    let mint_result = NonFungibleDomainClient::new(&e, &nfd(&e, None).unwrap()).try_mint(&to, token_id, node);
    if mint_result.is_err() {
        Err(RegistryV2Errors::MintingTokenFailed)
    } else {
        Ok(())
    }
}

pub fn burn_token(e: &Env, token_id: &u32) -> Result<(), RegistryV2Errors> {
    let mint_result = NonFungibleDomainClient::new(&e, &nfd(&e, None).unwrap()).try_burn(&token_id);
    if mint_result.is_err() {
        Err(RegistryV2Errors::BurningTokenFailed)
    } else {
        Ok(())
    }
}

pub fn calculate_exp_date(e: &Env, periods: &u64) -> u64 {
    e.ledger().timestamp() + (3600 * 24 * 360 * periods)
}
