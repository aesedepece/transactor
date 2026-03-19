use crate::{
    errors::Error,
    movements::{BalanceHistory, Movement, MovementStatus, MovementType},
    transactions::{Transaction, TransactionType},
    types::*,
};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

/// The main data structure for holding account balances and good standing status.
///
/// The client ID is omitted here because this structure is meant to be indexed using that fields as
/// the indexed key in the context of an `AccountsSystem`.
#[derive(Clone, Debug, Default)]
pub struct Account {
    available_balance: Value,
    held_balance: Value,
    locked: bool,
    balance_history: BalanceHistory,
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
    pub fn process_transaction(&mut self, transaction: &Transaction) -> Result<&Account, Error> {
        // Before anything else, we must check that the account is in a good state, i.e. it is not
        // locked. It is assumed that transactions cannot be processed for a locked account.
        // As a consequence of controlling locked / good state here, lower-level functions like
        // `process_deposit` are not guarded against locks; but this decision is deliberate and
        // harmless because those functions are private and the only real way to mutate accounts
        // from runtime is through this present `process_transaction` function.
        if self.is_in_good_state() {
            let tx = transaction;

            // Beyond any balance changes or any other changes to the account state, processing a
            // deposit or a withdrawal is expected to also return a movement that will later be
            // pushed into the account's balance history.
            let result = match tx.transaction_type {
                TransactionType::Deposit => self.deposit(tx.amount).map(Some),
                TransactionType::Withdrawal => self.withdraw(tx.amount).map(Some),
                TransactionType::Dispute => {
                    self.dispute(tx.client_id, tx.transaction_id).map(|_| None)
                }
                TransactionType::Resolve => {
                    self.resolve(tx.client_id, tx.transaction_id).map(|_| None)
                }
                TransactionType::Chargeback => self
                    .charge_back(tx.client_id, tx.transaction_id)
                    .map(|_| None),
            };

            // If processing the transaction resulted in the creation of a new movement (it was
            // either a deposit or a withdrawal), it is time to add it to the account's balance
            // history.
            if let Ok(Some(movement)) = result {
                self.balance_history.push(tx.transaction_id, movement);
            }

            result.map(|_| &*self)
        } else {
            Err(Error::LockedAccount)
        }
    }

    /// Process a deposit.
    ///
    /// Simply adds an amount of monetary value into the available balance of the account.
    ///
    /// Returns a new movement entry derived from the deposit facts and the former account balances.
    fn deposit(&mut self, amount: Option<Value>) -> Result<Movement, Error> {
        if let Some(amount) = amount {
            // Early return if amount is a negative number because... what does a negative deposit
            // even mean!?!?
            if amount <= Value::ZERO {
                return Err(Error::ZeroOrNegativeAmount(amount));
            }

            // All good, the available balance can be incremented
            self.available_balance += amount;

            // Derive movement from transaction
            let movement = Movement::new(MovementType::Deposit, amount, self.available_balance);

            Ok(movement)
        } else {
            Err(Error::DepositWithoutAmount)
        }
    }

    /// Process a withdrawal.
    ///
    /// Simply removes an amount of monetary value from the available balance of the account.
    ///
    /// Returns a new movement entry derived from the withdrawal facts and the former account
    /// balances.
    fn withdraw(&mut self, amount: Option<Value>) -> Result<Movement, Error> {
        if let Some(withdrawing) = amount {
            // Early return if amount is a negative number because... what does a negative
            // withdrawal even mean!?!?
            if withdrawing <= Value::ZERO {
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

            // Derive movement from transaction
            let movement = Movement::new(
                MovementType::Withdrawal,
                withdrawing,
                self.available_balance,
            );

            Ok(movement)
        } else {
            Err(Error::WithdrawalWithoutAmount)
        }
    }

    /// Process a dispute claim.
    ///
    /// Simply searches for an existing movement matching the transaction ID, tries to update its
    /// status from `InForce` into `Disputed`, and moves the corresponding amount into the held
    /// balance.
    fn dispute(&mut self, client_id: ClientId, transaction_id: TransactionId) -> Result<(), Error> {
        // Scoping is used here to prevent double-borrowing of self, leveraging the fact that
        // `Value` is `Copy` and hence can safely scape the scope through implicit returning.
        let (movement_type, movement_amount) = {
            // Try to find the movement referred by the transaction ID from the dispute claim
            let movement = self.balance_history.get_mut(&transaction_id).ok_or(
                Error::DisputingUnknownTransaction {
                    transaction: transaction_id,
                    client: client_id,
                },
            )?;

            // Trigger the status change on the original movement referred by the dispute claim
            movement.update_status(MovementStatus::Disputed)?;

            (movement.movement_type, movement.amount)
        };

        // All good, now let's put the balance on hold:
        // - If the disputed transaction was a deposit, we need to transfer from the available
        //   balance into the held balance.
        // - If the transaction was a withdrawal, the held balance increases, seemingly out of
        //   nowhere.
        // This last point apparently contradicts the requirement that total balance should remain
        // unchanged upon disputing, but there is simply no other way if we want to allow disputing
        // withdrawals: if by definition a withdrawal "destroys" balance (reduces the total amount
        // of balance that exists at a system level), then reverting a withdrawal must create
        // balance out of thin air, just like deposits do.
        match movement_type {
            MovementType::Deposit => {
                self.transfer_from_available_to_held(movement_amount)?;
            }
            MovementType::Withdrawal => {
                self.held_balance += movement_amount;
            }
        }

        Ok(())
    }

    /// Process the resolution of a former dispute.
    ///
    /// Simply searches for an existing movement matching the transaction ID, and tries to update
    /// its status from `Disputed` into `InForce`, and reverts the dispute (honors the original
    /// transaction).
    fn resolve(&mut self, client_id: ClientId, transaction_id: TransactionId) -> Result<(), Error> {
        // Scoping is used here to prevent double-borrowing of self, leveraging the fact that
        // `MovementType` and `Value` are `Copy` and hence can safely scape the scope through
        // implicit returning.
        let (movement_type, movement_amount) = {
            // Try to find the movement referred by the transaction ID from the resolution
            let movement = self.balance_history.get_mut(&transaction_id).ok_or(
                Error::ResolvingUnknownTransaction {
                    transaction: transaction_id,
                    client: client_id,
                },
            )?;

            // Trigger the status change on the original movement referred by the resolution
            movement.update_status(MovementStatus::InForce)?;

            (movement.movement_type, movement.amount)
        };

        // All good, move the balances around so the account balances are reverted to how they were
        // before the dispute:
        // - If the disputed (and now resolved) transaction was a deposit, we honor it by means of
        //   releasing the held balance back into the available balance.
        // - If the transaction was a withdrawal, we honor it by actually removing the withdrawn
        //   balance from the held balance.
        match movement_type {
            MovementType::Deposit => {
                self.transfer_from_held_to_available(movement_amount);
            }
            MovementType::Withdrawal => {
                self.held_balance -= movement_amount;
            }
        }

        Ok(())
    }

    /// Process the chargeback of a former dispute.
    ///
    /// Simply searches for an existing movement matching the transaction ID, tries to update
    /// its status from `Disputed` into `ChargedBack`, and executes the dispute (reverts the
    /// original transaction).
    ///
    /// Additionally, it locks the account.
    fn charge_back(
        &mut self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Result<(), Error> {
        // Scoping is used here to prevent double-borrowing of self, leveraging the fact that
        // `MovementType` and `Value` are `Copy` and hence can safely scape the scope through
        // implicit returning.
        let (movement_type, movement_amount) = {
            // Try to find the movement referred by the transaction ID from the resolution
            let movement = self.balance_history.get_mut(&transaction_id).ok_or(
                Error::ChargingBackUnknownTransaction {
                    transaction: transaction_id,
                    client: client_id,
                },
            )?;

            // Trigger the status change on the original movement referred by the resolution
            movement.update_status(MovementStatus::ChargedBack)?;

            (movement.movement_type, movement.amount)
        };

        // All good, move the balances around so the account balances are reverted to how they were
        // before the original transaction:
        // - If the disputed (and now charged back) transaction was a deposit, we revert it by means
        //   of removing its amount from the held balance.
        // - If the transaction was a withdrawal, we revert it by actually crediting the withdrawn
        //   balance into the available balance (transferring from held to available).
        match movement_type {
            MovementType::Deposit => {
                self.held_balance -= movement_amount;
            }
            MovementType::Withdrawal => {
                self.transfer_from_held_to_available(movement_amount);
            }
        }

        // Finally, lock the account.
        self.lock();

        Ok(())
    }

    /// Flag an account as locked.
    ///
    /// All further transactions affecting this account will be rejected until unlocked.
    ///
    /// This operation is infallible and idempotent: locking will set the locked state to `true`
    /// regardless of the previous state.
    #[inline]
    fn lock(&mut self) {
        self.locked = true;
    }

    /// Remove the lock from an account.
    ///
    /// This opens the door to further transactions being applied on this account.
    ///
    /// This operation is infallible and idempotent: unlocking will set the locked state to `false`
    /// regardless of the previous state.
    ///
    /// As per the scope of this PoC, unlocking is not possible in runtime, so this is implemented
    /// only for testing purposes, hence why it is guarded behind `#[cfg(test)]`.
    #[cfg(test)]
    #[inline]
    fn unlock(&mut self) {
        self.locked = false;
    }

    /// Move the specified amount of monetary value from the available balance into the held
    /// balance.
    ///
    /// This balance change is only expected to happen upon disputing a deposit.
    ///
    /// *Note*: trying to put on hold more balance that the current available balance will fail.
    /// Please read the comments in the source code for the rationale.
    // The requirements do not specify how to handle the situation in which a deposit is disputed,
    // but the monetary value that it brought was spent ever since, leasing to an available balance
    // that is lower than the amount that we want to put on hold.
    //
    // As a consequence, a business decision needs to be made here, where the two apparent ways of
    // approaching this situation are:
    //
    // 1. Capping the amount to be put on hold to the current available balance, i.e. if we try to
    //    put 50 units of monetary value on hold, but our available balance is only 30 units, the
    //    available balance becomes 0 and the held balance becomes 30;
    // 2. Allow balances to go negative, i.e. if we try to put 50 units of monetary value on hold,
    //    but our available balance is only 30 units, the available balance becomes -20 and the held
    //    balance becomes 50;
    // 3. Throw an error and refuse to process the dispute, while balances remain unchanged.
    //
    // It is an explicit requirement that upon putting balance on hold, the total balance (the
    // aggregate of available balance and held balance) should not change. In order to explicitly
    // honor this requirement, the approach numbered "1)" above is discarded.
    //
    // However, the requirements namely state that in case of doubt, assumptions must be made such
    // that they make sense for an ATM or a bank. Therefore, I deem adequate to take the most
    // conservative and "pesimistic" approach, which will provide the guarantees that better
    // adequate to ACID principles. That is, of course, the approach numbered 3).
    fn transfer_from_available_to_held(&mut self, amount: Value) -> Result<(), Error> {
        // Early return if hitting the undefined behavior described above
        if amount > self.available_balance {
            return Err(Error::DisputeAmountExceedsAvailableBalance {
                disputing: amount,
                available: self.available_balance,
            });
        }

        // All good, we can perform the transfer
        self.available_balance -= amount;
        self.held_balance += amount;

        Ok(())
    }

    /// Move the specified amount of monetary value from the held balance into the available
    /// balance.
    ///
    /// This balance change is only expected to happen upon resolving a disputed withdrawal.
    ///
    /// *Note*: this does not suffer from the unspecified behavior affecting the opposite
    /// `transfer_from_available_to_held()` function, because held balance can only come from
    /// successfully disputing a former deposit or withdrawal. In other words, this function should
    /// be infallible in practice.
    fn transfer_from_held_to_available(&mut self, amount: Value) {
        self.held_balance -= amount;
        self.available_balance += amount;
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
    fn process_transaction(&mut self, transaction: &Transaction) -> Result<&Account, Error> {
        // The entry API makes it convenient, efficient and safe to "upsert" account entries into
        // our system, i.e. creating the entry if it does not exist before even trying to process
        // the transaction.
        // In this case, it is fine to initialize the account entry to its default value (balances
        // set to zero, no lock, no history).
        self.get_account_mut(transaction.client_id)
            .process_transaction(transaction)
    }
}
