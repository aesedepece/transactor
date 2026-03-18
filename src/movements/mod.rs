use crate::{errors::Error, types::*};
use indexmap::IndexMap;

#[cfg(test)]
mod test;

/// The different types of movements to track for account balance histories.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MovementType {
    /// A balance increment, derived from a deposit transaction.
    Deposit,
    /// A balance decrement, derived from a withdrawal transaction.
    Withdrawal,
}

/// The different statuses that a movement can go through.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum MovementStatus {
    /// By default, a movement is "in force", i.e. it has not been disputed, resolved or charged
    /// back.
    #[default]
    InForce,
    /// A movement that has been disputed upon an user's claim. This will eventually progress
    /// `ChargedBack` if the dispute progresses; or back to `InForce` if the dispute is rejected
    /// (namely, by processing a `Transaction::Resolve`).
    Disputed,
    /// The movement has been reverted upon processing a first `Transaction::Dispute` and a
    /// subsequent `Transaction::ChargeBack`.
    ChargedBack,
}

/// Movements are derived from transactions and conform the actual balance history of an account.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Movement {
    /// Which type of movement is this.
    movement_type: MovementType,
    /// The amount of monetary value that the movement is depositing or withdrawing.
    amount: Value,
    /// The final available balance after this movement is applied.
    available_balance_after: Value,
    /// The current status of this movement.
    status: MovementStatus,
}

impl Movement {
    /// Create a new movement, most likely from the result of processing a deposit or a withdrawal
    /// transaction.
    #[inline]
    pub fn new(movement_type: MovementType, amount: Value, available_balance_after: Value) -> Self {
        Movement {
            movement_type,
            amount,
            available_balance_after,
            status: Default::default(),
        }
    }

    /// Updates the status of the movement following a strict state machine that forbids
    /// non-compliant transitions.
    pub fn update_status(&mut self, new_status: MovementStatus) -> Result<(), Error> {
        // Local `use` of `enum` members to ease readability of the `match` below
        use MovementStatus::*;
        let from_status = self.status;

        match (from_status, new_status) {
            // Changing from one status to the same status is always legal, but does nothing!
            (x, y) if x == y => {
              Ok(())
            },
            // Movements that are in force cannot be charged back without being disputed first
            (InForce, ChargedBack)
            // Movements that are charged back cannot change status anymore
            | (ChargedBack, _) => {
                Err(Error::IllegalMovementStatusTransition {
                    from: from_status,
                    to: new_status
                })
            },
            // All other transitions are legal and trigger an actual mutation
            (_, new_status) => {
                self.status = new_status;

                Ok(())
            }
        }
    }
}

/// An efficient way to track an account's balance history.
///
/// Internal usage of `IndexMap` guarantees that we can fetch the entire history in the original
/// order of insertion, and individual entries queried by ID, both with `O(1)` complexity.
#[derive(Clone, Debug, Default)]
pub struct BalanceHistory {
    history: IndexMap<TransactionId, Movement>,
}

impl BalanceHistory {
    /// Tells whether the history is empty.
    // In the Rust ecosystem, it is customary to implement a function like this every time that you
    // implement a `len()` function.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Tells how many movement entries exist in the balance history.
    #[inline]
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Extracts the balance history as a vector of tuples where the first item is the transaction
    /// ID of the transaction associated with the movement, and the second item is the movement
    /// itself.
    #[inline]
    pub fn to_vec(&self) -> Vec<(&TransactionId, &Movement)> {
        self.history.iter().collect()
    }

    #[inline]
    /// Pushes / inserts a new movement into the history.
    ///
    /// In the event that a movement with the same transaction ID was added before, it will simply
    /// update the movement data.
    pub fn push(&mut self, id: TransactionId, movement: Movement) {
        self.history.insert(id, movement);
    }

    /// Obtains a reference to an existing movement that is already present in the history, as
    /// identified by its transaction ID.
    #[inline]
    pub fn get(&self, id: &TransactionId) -> Option<&Movement> {
        self.history.get(id)
    }
}
