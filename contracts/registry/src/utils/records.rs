use crate::errors::ContractErrors;
use soroban_sdk::{panic_with_error, symbol_short, Address, Bytes, BytesN, Env};

mod oracle {
    soroban_sdk::contractimport!(file = "../../oracle.wasm");
}

// Currently there are forbidden types of domains:
// - Domains with numbers
// - Domains with special characters
// - Domains with uppercase letters
// - Domains that are longer than 15 characters
pub fn validate_domain(e: &Env, domain: &Bytes) {
    if domain.len() > 15 {
        panic_with_error!(&e, &ContractErrors::InvalidDomain);
    }

    for byte in domain.iter() {
        if byte < 97 || byte > 122 {
            panic_with_error!(&e, &ContractErrors::InvalidDomain);
        }
    }
}

// This function is used to generate the nodes based on the "domain" and the "parent".
// The parent can be either the root domain (in the case of generating a subdomain node) or the TLD (when generating a root
// domain node).
pub fn generate_node(e: &Env, domain: &Bytes, parent: &Bytes) -> BytesN<32> {
    let parent_hash: BytesN<32> = e.crypto().keccak256(&parent).to_bytes();
    let domain_hash: BytesN<32> = e.crypto().keccak256(&domain).to_bytes();

    let mut node_builder: Bytes = Bytes::new(&e);
    node_builder.append(&Bytes::from(&parent_hash));
    node_builder.append(&Bytes::from(&domain_hash));

    e.crypto().keccak256(&node_builder).to_bytes()
}

// This calculates how much will it costs to set a new domain based on its length
// It first defines the price in USD and, then it calculates the amount of collateral to request
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
    let rate_price: u128 = oracle_client
        .lastprice(&oracle::Asset::Other(symbol_short!("XLM")))
        .unwrap()
        .price as u128;

    let collateral_price: u128 = if decimals > 7 {
        rate_price / 10u128.pow(decimals - 7)
    } else {
        rate_price
    };

    let collateral_amount: u128 = (usd_value * 10u128.pow(7)) / collateral_price;

    (usd_value, collateral_amount)
}

#[cfg(test)]
mod test_records_utils {
    use crate::utils::records::record_price;
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::{symbol_short, Address, Env, String};

    mod oracle {
        soroban_sdk::contractimport!(file = "../../oracle.wasm");
    }

    #[test]
    fn test_record_price() {
        let e: Env = Env::from_ledger_snapshot_file("../../network_snapshots/reflector.json");
        e.ledger().set_protocol_version(21);
        e.ledger().set_timestamp(1742825701);
        let oracle_addr: Address = Address::from_string(&String::from_str(
            &e,
            "CAFJZQWSED6YAWZU3GWRTOCNPPCGBN32L7QV43XX5LZLFTK6JLN34DLN",
        ));

        let oracle_client: oracle::Client = oracle::Client::new(&e, &oracle_addr);

        // 0.2919892
        let last: i128 = oracle_client
            .price(&oracle::Asset::Other(symbol_short!("XLM")), &1742825700)
            .unwrap()
            .price
            / 10i128.pow(oracle_client.decimals() - 7);

        for i in 1..6 {
            e.budget().reset_default();
            if i == 1 {
                let (usd_value, collateral_amount) = record_price(&e, &oracle_addr, i);
                assert_eq!(usd_value, 187_5700000);
                assert_eq!(collateral_amount, 642_3867732);
            } else if i == 2 {
                let (usd_value, collateral_amount) = record_price(&e, &oracle_addr, i);
                assert_eq!(usd_value, 107_1800000);
                assert_eq!(collateral_amount, 367_0683710);
            } else if i == 3 {
                let (usd_value, collateral_amount) = record_price(&e, &oracle_addr, i);
                assert_eq!(usd_value, 61_2500000);
                assert_eq!(collateral_amount, 209_7680325);
            } else if i == 4 {
                let (usd_value, collateral_amount) = record_price(&e, &oracle_addr, i);
                assert_eq!(usd_value, 35_0000000);
                assert_eq!(collateral_amount, 119_8674471);
            } else {
                let (usd_value, collateral_amount) = record_price(&e, &oracle_addr, i);
                assert_eq!(usd_value, 20_0000000);
                assert_eq!(collateral_amount, 68_4956840);
            }
        }
    }
}
