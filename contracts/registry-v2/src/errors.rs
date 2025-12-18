use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum RegistryV2Errors {
    UnexpectedError = 300,
    InvalidDomain = 301,
    InvalidSubDomain = 302,
    UnsupportedTLD = 303,
    DomainAlreadyExist = 304,
    PaymentFailed = 305,
    MintingTokenFailed = 306,
    RecordDoesntExist = 307,
    RecordIsExpired = 308,
    BurningTokenFailed = 309,
    RecordCantBeClaimedYet = 310,
    V1DomainRegistered = 311,
    InvalidV1Domain = 312,
    V1DomainMigrationExpired = 313,
}
