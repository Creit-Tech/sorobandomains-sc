use soroban_sdk::{contracttype, Bytes, Vec};

// Represents a domain structure with a top-level domain (TLD),
// second-level domain (SLD), and a vector of subdomains.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Domain {
    // The top-level domain (TLD), e.g., "xlm" in `a.b.example.xlm`.
    pub tld: Bytes,

    // The second-level domain (SLD), e.g., "example" in `a.b.example.xlm`.
    pub sld: Bytes,

    // A list of subdomains, e.g., ["a", "b"] in `a.b.example.xlm`.
    // Currently, the maximum number of subdomains is 1, but more can be added in the future.
    // Keep it empty if there are no subdomains.
    pub subs: Vec<Bytes>,
}

#[contracttype]
pub enum CoreDataKeys {
    Admin,
    Registry,
    Fee,
    Currency,
    Treasury,
}
