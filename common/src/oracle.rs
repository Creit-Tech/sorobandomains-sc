use soroban_sdk::{symbol_short, Address, Env};
mod oracle {
    soroban_sdk::contractimport!(file = "../oracle.wasm");
}

/// This calculates how much will it costs to set a new domain based on its length
/// It first defines the price in USD and, then it calculates the amount of collateral to request
pub fn record_price(e: &Env, oracle_addr: &Address, length: u32) -> (u128, u128) {
    let usd_value: u128 = if length >= 5 {
        20_0000000
    } else if length == 4 {
        35_0000000
    } else if length == 3 {
        61_2500000
    } else if length == 2 {
        107_1800000
    } else {
        187_5700000
    };

    let oracle_client: oracle::Client = oracle::Client::new(&e, oracle_addr);
    let decimals: u32 = oracle_client.decimals();
    let rate_price: u128 = oracle_client.lastprice(&oracle::Asset::Other(symbol_short!("XLM"))).unwrap().price as u128;

    let xlm_price: u128 = if decimals > 7 {
        rate_price / 10u128.pow(decimals - 7)
    } else {
        rate_price
    };

    let xlm_amount: u128 = (usd_value * 10u128.pow(7)) / xlm_price;

    (usd_value, xlm_amount)
}
