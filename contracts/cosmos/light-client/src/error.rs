use cosmwasm_std::StdError;
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError{
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("A wild caveman has appeared, screaming {0:?}")]
    CavemanError(String),

    #[error("{0}")]
    InvalidHex(FromHexError),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

#[derive(Error, Debug)]
pub enum FixedVecError {
    #[error("OutOfBounds")]
    OutOfBounds {
        i: usize,
        len: usize,
    },
    /// A `BitList` does not have a set bit, therefore it's length is unknowable.
    #[error("MissingLengthInformation")]

    MissingLengthInformation,
    /// A `BitList` has excess bits set to true.
    #[error("ExcessBits")]
    ExcessBits,
    /// A `BitList` has an invalid number of bytes for a given bit length.
    #[error("InvalidByteCount")]
    InvalidByteCount {
        given: usize,
        expected: usize,
    },
}
