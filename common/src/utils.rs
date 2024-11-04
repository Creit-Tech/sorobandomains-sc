use soroban_sdk::{Bytes, BytesN, Env};

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
