use crate::errors::ContractErrors;
use soroban_sdk::{panic_with_error, Bytes, BytesN, Env};

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
    let parent_hash: BytesN<32> = e.crypto().keccak256(&parent);
    let domain_hash: BytesN<32> = e.crypto().keccak256(&domain);

    let mut node_builder: Bytes = Bytes::new(&e);
    node_builder.append(&Bytes::from(&parent_hash));
    node_builder.append(&Bytes::from(&domain_hash));

    e.crypto().keccak256(&node_builder)
}
