use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractErrors {
    UnexpectedError = 0,
    FailedToGetRecord = 1,
    FeePaymentFailed = 2,
    KeyWasInvalidated = 3,
}
