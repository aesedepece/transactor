use crate::{movements::MovementStatus, types::*};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("tried to charge back an unknown transaction id ({transaction}) for client ({client})")]
    ChargingBackUnknownTransaction {
        transaction: TransactionId,
        client: ClientId,
    },
    #[error("tried to process a deposit without specifying an amount")]
    DepositWithoutAmount,
    #[error(
        "tried to dispute more value ({disputing}) than the current available balance ({available})"
    )]
    DisputeAmountExceedsAvailableBalance { disputing: Value, available: Value },
    #[error("tried to dispute an unknown transaction id ({transaction}) for client ({client})")]
    DisputingUnknownTransaction {
        transaction: TransactionId,
        client: ClientId,
    },
    #[error("illegal movement status transition from {from:?} to {to:?}")]
    IllegalMovementStatusTransition {
        from: MovementStatus,
        to: MovementStatus,
    },
    #[error("tried to process a transaction for an account that is locked")]
    LockedAccount,
    #[error("tried to resolve an unknown transaction id ({transaction}) for client ({client})")]
    ResolvingUnknownTransaction {
        transaction: TransactionId,
        client: ClientId,
    },
    #[error("unknown or indeterminate error: {_0}")]
    UnknownError(String),
    #[error(
        "tried to withdraw more value ({withdrawing}) than the current available balance ({available})"
    )]
    WithdrawalAmountExceedsAvailableBalance {
        withdrawing: Value,
        available: Value,
    },
    #[error("tried to process a withdrawal without specifying an amount")]
    WithdrawalWithoutAmount,
    #[error("cannot process transactions with a zero or negative amount ({_0})")]
    ZeroOrNegativeAmount(Value),
}
