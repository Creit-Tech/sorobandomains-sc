use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractErrors {
    AlreadyStarted = 1,
    RecordAlreadyExist = 2,
    InvalidDuration = 3,
    UnsupportedTLD = 4,
    RecordDoesntExist = 5,
    InvalidDomain = 6,
    ExpiredDomain = 7,
    InvalidParent = 8,
    OutdatedSub = 9,
}
