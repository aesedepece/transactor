use crate::{movements::MovementStatus, types::*};

/// An enumeration of all the possible errors that our engine can encounter.
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    /// Charging back an unknown transaction.
    #[error("tried to charge back an unknown transaction id ({transaction}) for client ({client})")]
    ChargingBackUnknownTransaction {
        /// The ID of the transaction to be charged back.
        transaction: TransactionId,
        /// The ID of the client initiating the transaction to be charged back.
        client: ClientId,
    },
    /// Wrapper for all kinds of [CSV errors](https://docs.rs/csv/latest/csv/struct.Error.html).
    #[error("a CSV error occurred: {message}")]
    CsvError {
        /// Error message for the CSV error.
        message: String,
    },
    /// Processing a deposit that specifies no amount.
    #[error("tried to process a deposit without specifying an amount")]
    DepositWithoutAmount,
    /// Disputing more value than is available currently for a certain account.
    #[error(
        "tried to dispute more value ({disputing}) than the current available balance ({available})"
    )]
    DisputeAmountExceedsAvailableBalance {
        /// The amount of monetary units that are being disputed.
        disputing: Value,
        /// The amount of monetary units currently available in the account.
        available: Value,
    },
    /// Disputing a transaction that is unknown for a certain account.
    #[error("tried to dispute an unknown transaction id ({transaction}) for client ({client})")]
    DisputingUnknownTransaction {
        /// The ID of the transaction to be disputed.
        transaction: TransactionId,
        /// The ID of the client initiating the transaction to be disputed.
        client: ClientId,
    },
    /// Performing a non-compliant movement status transition (e.g. charging back a transaction that
    /// has not been disputed yet).
    #[error("illegal movement status transition from {from:?} to {to:?}")]
    IllegalMovementStatusTransition {
        /// The current status.
        from: MovementStatus,
        /// The status into which the transaction is trying to transition.
        to: MovementStatus,
    },
    /// Wrapper for all kinds of [IO errors](https://doc.rust-lang.org/std/io/struct.Error.html).
    #[error("an IO error of kind \"{kind}\" occurred: {message}")]
    IoError {
        /// The [kind](https://doc.rust-lang.org/std/io/enum.ErrorKind.html) of IO error.
        kind: std::io::ErrorKind,
        /// Error mesage for the IO error.
        message: String,
    },
    /// Processing a transaction on a locked account.
    #[error("tried to process a transaction for an account that is locked")]
    LockedAccount,
    /// Resolving a transaction that is unknown for a certain account.
    #[error("tried to resolve an unknown transaction id ({transaction}) for client ({client})")]
    ResolvingUnknownTransaction {
        /// The ID of the transaction to be resolved.
        transaction: TransactionId,
        /// The ID of the client who initiated the transaction to be resolved.
        client: ClientId,
    },
    /// Conflates any other type of error not covered in this enum.
    #[error("unknown or indeterminate error: {_0}")]
    UnknownError(String),
    /// Withdrawing more monetary value than is currently available for a certain account.
    #[error(
        "tried to withdraw more value ({withdrawing}) than the current available balance ({available})"
    )]
    WithdrawalAmountExceedsAvailableBalance {
        /// The amount of monetary units to be withdrawn.
        withdrawing: Value,
        /// The amount of monetary units currently available for the account.
        available: Value,
    },
    /// Processing a withdrawal that specifies no amount.
    #[error("tried to process a withdrawal without specifying an amount")]
    WithdrawalWithoutAmount,
    /// Processing a transaction that specifies a zero or negative amount.
    #[error("cannot process transactions with a zero or negative amount ({_0})")]
    ZeroOrNegativeAmount(Value),
}

impl From<std::io::Error> for Error {
    fn from(internal: std::io::Error) -> Self {
        Error::IoError {
            kind: internal.kind(),
            message: internal.to_string(),
        }
    }
}

impl From<csv::Error> for Error {
    fn from(internal: csv::Error) -> Self {
        Error::CsvError {
            message: internal.to_string(),
        }
    }
}
