use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    FailedToGetRecord = 1,
    AddressMismatch = 2,
    NotImplemented = 3,
    FailedToPayFee = 4,
}
