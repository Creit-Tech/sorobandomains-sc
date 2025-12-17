use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum NonFungibleTokenError {
    NonExistentToken = 200,
    IncorrectOwner = 201,
    InsufficientApproval = 202,
    InvalidApprover = 203,
    InvalidLiveUntilLedger = 204,
    TokenIdAlreadyExist = 225,
}
