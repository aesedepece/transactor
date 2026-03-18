use crate::{
    errors::Error,
    transactions::{Transaction, TransactionType},
    types::*,
};
use num::traits::Zero;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

/// The main data structure for holding account balances and good standing status.
///
/// The client ID is omitted here because this structure is meant to be indexed using that fields as
/// the indexed key in the context of an `AccountsSystem`.
// TODO: add a history field that allows recovering transaction info later if needed for disputes or
//  traceability
#[derive(Copy, Clone, Debug, Default)]
pub struct Account {
    available_balance: Value,
    held_balance: Value,
    locked: bool,
}

impl Account {
    /// Convenience method for idiomatically checking if an account is in good state.
    #[inline]
    pub fn is_in_good_state(&self) -> bool {
        !self.locked
    }

    /// Apply any type of transaction on an account, internally mutating it as expected from the
    /// semantics of the transaction type.
    ///
    /// Upon success, returns the final state of the account, i.e. how it looks like after mutation.
    pub fn process_transaction(&mut self, transaction: &Transaction) -> Result<Account, Error> {
        // Before anything else, we must check that the account is in a good state, i.e. it is not
        // locked. It is assumed that transactions cannot be processed for a locked account.
        // As a consequence of controlling locked / good state here, lower-level functions like
        // `process_deposit` are not guarded against locks; but this decision is deliberate and
        // harmless because those functions are private and the only real way to mutate accounts
        // from runtime is through this present `process_transaction` function.
        if self.is_in_good_state() {
            let tx = transaction;

            let result = match tx.transaction_type {
                TransactionType::Deposit => self.deposit(tx.amount),
                TransactionType::Withdrawal => self.withdraw(tx.amount),
                TransactionType::Dispute => Ok(()),
                TransactionType::Resolve => Ok(()),
                TransactionType::Chargeback => Ok(()),
            };

            // TODO: push the transaction into the account's transaction history, allowing to
            //  recover its info later if needed for disputes or traceability
            if result.is_ok() {}

            result.map(|_| *self)
        } else {
            Err(Error::LockedAccount)
        }
    }

    /// Process a deposit.
    ///
    /// Simply adds an amount of monetary value into the available balance of the account.
    fn deposit(&mut self, amount: Option<Value>) -> Result<(), Error> {
        if let Some(amount) = amount {
            // Early return if amount is a negative number because... what does a negative deposit
            // even mean!?!?
            if amount <= Value::zero() {
                return Err(Error::ZeroOrNegativeAmount(amount));
            }

            // All good, the available balance can be incremented
            self.available_balance += amount;

            Ok(())
        } else {
            Err(Error::DepositWithoutAmount)
        }
    }

    /// Process a withdrawal.
    ///
    /// Simply removes an amount of monetary value from the available balance of the account.
    fn withdraw(&mut self, amount: Option<Value>) -> Result<(), Error> {
        if let Some(withdrawing) = amount {
            // Early return if amount is a negative number because... what does a negative
            // withdrawal even mean!?!?
            if withdrawing <= Value::zero() {
                return Err(Error::ZeroOrNegativeAmount(withdrawing));
            }

            // Return early if trying to withdraw more value than the available balance
            let available = self.available_balance;
            if withdrawing > available {
                return Err(Error::WithdrawalAmountExceedsAvailableBalance {
                    withdrawing,
                    available,
                });
            }

            // All good, the available balance can be decremented
            self.available_balance = available - withdrawing;

            Ok(())
        } else {
            Err(Error::WithdrawalWithoutAmount)
        }
    }

    /// Flag an account as locked.
    ///
    /// All further transactions affecting this account will be rejected until unlocked.
    ///
    /// This operation is infallible and idempotent: locking will set the locked state to `true`
    /// regardless of the previous state.
    #[inline]
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Remove the lock from an account.
    ///
    /// This opens the door to further transactions being applied on this account.
    ///
    /// This operation is infallible and idempotent: unlocking will set the locked state to `false`
    /// regardless of the previous state.
    #[inline]
    pub fn unlock(&mut self) {
        self.locked = false;
    }
}

/// A collection of accounts that represents the whole userbase of the system.
///
/// Account entries are implicitly added upon processing a first transaction affecting such client
/// ID.
// The requirements do not specify whether accounts with zero balance should be automatically
// dropped. A decision was made to keep them in place for the sake of traceability and the
// ability to dispute a "total withdrawal" (when the user withdraws the totality of their balance,
// leaving zero value behind).
pub struct AccountsSystem {
    accounts: HashMap<ClientId, Account>,
}

impl AccountsSystem {
    /// Get an immutable reference to the account entry matching a client ID.
    #[inline]
    fn get_account(&self, id: ClientId) -> Option<&Account> {
        self.accounts.get(&id)
    }

    /// Get a mutable reference to the account entry matching a client ID.
    fn get_account_mut(&mut self, id: ClientId) -> &mut Account {
        self.accounts.entry(id).or_default()
    }

    /// Apply a transaction on an account allegedly contained in the accounts system, internally
    /// mutating it as expected from the semantics of the transaction type.
    ///
    /// Upon success, returns the final state of the account, i.e. how it looks like after mutation.
    fn process_transaction(&mut self, transaction: &Transaction) -> Result<Account, Error> {
        // The entry API makes it convenient, efficient and safe to "upsert" account entries into
        // our system, i.e. creating the entry if it does not exist before even trying to process
        // the transaction.
        // In this case, it is fine to initialize the account entry to its default value (balances
        // set to zero, no lock, no history).
        self.get_account_mut(transaction.client_id)
            .process_transaction(transaction)
    }
}
