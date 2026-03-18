use crate::{accounts::Account, errors::Error, types::*};
use num::traits::Zero;
use std::ops::Mul;

#[test]
fn account_initial_state() {
    let account = Account::default();

    // Initial available balance is zero
    assert_eq!(account.available_balance, Value::zero());
    // Initial held balance is zero
    assert_eq!(account.held_balance, Value::zero());
    // Account is not locked
    assert_eq!(account.is_in_good_state(), true);
}

#[test]
fn account_locking() {
    let mut account = Account::default();

    // Account is not locked
    assert_eq!(account.is_in_good_state(), true);

    // Unlocking an unlocked account does nothing
    account.unlock();
    assert_eq!(account.is_in_good_state(), true);

    // Locking an unlocked account effectively locks it
    account.lock();
    assert_eq!(account.is_in_good_state(), false);

    // Locking an already-locked account does nothing
    account.lock();
    assert_eq!(account.is_in_good_state(), false);

    // Accounts can be unlocked and back to a good state after being locked
    account.unlock();
    assert_eq!(account.is_in_good_state(), true);
}

#[test]
fn processing_deposits() {
    let mut account = Account::default();

    // An unlocked account can process a deposit with a positive amount, and the available balance
    // gets incremented accordingly
    let deposit_amount = Value::from(123.456);
    assert!(account.deposit(Some(deposit_amount)).is_ok());
    assert_eq!(account.available_balance, deposit_amount);

    // Further deposits can be made on the same account, and balance is added up
    assert!(account.deposit(Some(deposit_amount)).is_ok());
    assert_eq!(
        account.available_balance,
        deposit_amount.mul(Value::from(2.0))
    );

    // Deposits with no amount, negative amounts or zero amounts are not allowed
    assert_eq!(account.deposit(None), Err(Error::DepositWithoutAmount));
    assert_eq!(
        account.deposit(Some(Value::zero())),
        Err(Error::ZeroOrNegativeAmount(Value::zero()))
    );
    assert_eq!(
        account.deposit(Some(-deposit_amount)),
        Err(Error::ZeroOrNegativeAmount(-deposit_amount))
    );

    // Please note that deposits on locked accounts are not tested here because account locks
    // operate on a higher level.
    // Those are tested separately for the function `process_transaction()`.
    // For more color on why this is just fine, please check the comments in the body of function
    // `process_transaction()`.
    // Movement derivation is not tested here for a similar reason: it only happens at a higher
    // level where the system is aware of transactions and transaction IDs.
}

#[test]
fn processing_withdrawals() {
    let initial_amount = Value::from(1_000.0);
    let mut account = Account {
        available_balance: initial_amount,
        ..Default::default()
    };

    // An unlocked account can process a deposit with a positive amount, and the available balance
    // gets decremented accordingly
    let withdrawal_amount = Value::from(123.456);
    assert!(account.withdraw(Some(withdrawal_amount)).is_ok());
    assert_eq!(
        account.available_balance,
        initial_amount - withdrawal_amount
    );

    // Further deposits can be made on the same account, and balance is added up
    assert!(account.withdraw(Some(withdrawal_amount)).is_ok());
    assert_eq!(
        account.available_balance,
        initial_amount - withdrawal_amount.mul(Value::from(2.0))
    );

    // Withdrawal with no amount, negative amounts or zero amounts are not allowed
    assert_eq!(account.withdraw(None), Err(Error::WithdrawalWithoutAmount));
    assert_eq!(
        account.withdraw(Some(Value::zero())),
        Err(Error::ZeroOrNegativeAmount(Value::zero()))
    );
    assert_eq!(
        account.withdraw(Some(-withdrawal_amount)),
        Err(Error::ZeroOrNegativeAmount(-withdrawal_amount))
    );

    // Can not withdraw more than you have, you cheater!
    assert_eq!(
        account.withdraw(Some(initial_amount)),
        Err(Error::WithdrawalAmountExceedsAvailableBalance {
            available: initial_amount - withdrawal_amount.mul(Value::from(2.0)),
            withdrawing: initial_amount,
        })
    );

    // But you can sweep your account and leave it empty though
    assert!(
        account
            .withdraw(Some(
                initial_amount - withdrawal_amount.mul(Value::from(2.0))
            ))
            .is_ok()
    );
    assert_eq!(account.available_balance, Value::zero());

    // Please note that withdrawals on locked accounts are not tested here because account locks
    // operate on a higher level.
    // Those are tested separately for the function `process_transaction()`.
    // For more color on why this is just fine, please check the comments in the body of function
    // `process_transaction()`.
    // Movement derivation is not tested here for a similar reason: it only happens at a higher
    // level where the system is aware of transactions and transaction IDs.
}
