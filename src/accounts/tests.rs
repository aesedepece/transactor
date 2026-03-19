use crate::movements::MovementStatus::{ChargedBack, Disputed, InForce};
use crate::{accounts::Account, errors::Error, transactions::Transaction, types::*};
use std::ops::Mul;

#[test]
fn account_initial_state() {
    let account = Account::default();

    // Initial available balance is zero
    assert_eq!(account.available_balance, Value::ZERO);
    // Initial held balance is zero
    assert_eq!(account.held_balance, Value::ZERO);
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
fn processing_deposit_facts() {
    let mut account = Account::default();

    // An unlocked account can process a deposit with a positive amount, and the available balance
    // gets incremented accordingly
    let deposit_amount = Value::from_num(123.456);
    assert!(account.deposit(Some(deposit_amount)).is_ok());
    assert_eq!(account.available_balance, deposit_amount);

    // Further deposits can be made on the same account, and balance is added up
    assert!(account.deposit(Some(deposit_amount)).is_ok());
    assert_eq!(
        account.available_balance,
        deposit_amount.mul(Value::from_num(2.0))
    );

    // Deposits with no amount or zero amounts are not allowed
    assert_eq!(account.deposit(None), Err(Error::DepositWithoutAmount));
    assert_eq!(
        account.deposit(Some(Value::ZERO)),
        Err(Error::ZeroOrNegativeAmount(Value::ZERO))
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
fn processing_withdrawal_facts() {
    let initial_amount = Value::from_num(1_000.0);
    let mut account = Account {
        available_balance: initial_amount,
        ..Default::default()
    };

    // An unlocked account can process a deposit with a positive amount, and the available balance
    // gets decremented accordingly
    let withdrawal_amount = Value::from_num(123.456);
    assert!(account.withdraw(Some(withdrawal_amount)).is_ok());
    assert_eq!(
        account.available_balance,
        initial_amount - withdrawal_amount
    );

    // Further deposits can be made on the same account, and balance is added up
    assert!(account.withdraw(Some(withdrawal_amount)).is_ok());
    assert_eq!(
        account.available_balance,
        initial_amount - withdrawal_amount.mul(Value::from_num(2.0))
    );

    // Withdrawals with no amount or zero amounts are not allowed
    assert_eq!(account.withdraw(None), Err(Error::WithdrawalWithoutAmount));
    assert_eq!(
        account.withdraw(Some(Value::ZERO)),
        Err(Error::ZeroOrNegativeAmount(Value::ZERO))
    );

    // Can not withdraw more than you have, you cheater!
    assert_eq!(
        account.withdraw(Some(initial_amount)),
        Err(Error::WithdrawalAmountExceedsAvailableBalance {
            available: initial_amount - withdrawal_amount.mul(Value::from_num(2.0)),
            withdrawing: initial_amount,
        })
    );

    // But you can sweep your account and leave it empty though
    assert!(
        account
            .withdraw(Some(
                initial_amount - withdrawal_amount.mul(Value::from_num(2.0))
            ))
            .is_ok()
    );
    assert_eq!(account.available_balance, Value::ZERO);

    // Please note that withdrawals on locked accounts are not tested here because account locks
    // operate on a higher level.
    // Those are tested down below at `processing_withdrawal_transactions()`.
    // For more color on why this is just fine, please check the comments in the body of function
    // `process_transaction()`.
    // Movement derivation is not tested here for a similar reason: it only happens at a higher
    // level where the system is aware of transactions and transaction IDs.
}

#[test]
fn processing_transactions_on_account() {
    use crate::transactions::TransactionType::*;

    let mut account = Account::default();

    // An unlocked account can process a deposit with a positive amount, and the available balance
    // gets incremented accordingly
    let first_deposit_tx_id = TransactionId::from(1u8);
    let first_deposit_amount = Value::from_num(123.456);
    let first_deposit = Transaction {
        transaction_type: Deposit,
        client_id: Default::default(),
        transaction_id: first_deposit_tx_id,
        amount: Some(first_deposit_amount),
    };
    assert!(matches!(account.process_transaction(&first_deposit), Ok(_)));
    assert_eq!(account.available_balance, first_deposit_amount);
    assert_eq!(account.held_balance, Value::ZERO);

    // A deposit cannot be resolved before being disputed first
    let resolve = Transaction {
        transaction_type: Resolve,
        client_id: Default::default(),
        transaction_id: first_deposit_tx_id,
        amount: None,
    };
    assert_eq!(
        account.process_transaction(&resolve).unwrap_err(),
        Error::IllegalMovementStatusTransition {
            from: InForce,
            to: InForce,
        }
    );
    assert_eq!(account.available_balance, first_deposit_amount);
    assert_eq!(account.held_balance, Value::ZERO);

    // A deposit cannot be charged back before being disputed first
    let chargeback = Transaction {
        transaction_type: Chargeback,
        client_id: Default::default(),
        transaction_id: first_deposit_tx_id,
        amount: None,
    };
    assert_eq!(
        account.process_transaction(&chargeback).unwrap_err(),
        Error::IllegalMovementStatusTransition {
            from: InForce,
            to: ChargedBack,
        }
    );
    assert_eq!(account.available_balance, first_deposit_amount);
    assert_eq!(account.held_balance, Value::ZERO);

    // An unlocked account can process a withdrawal with a positive amount, and the available
    // balance gets decremented accordingly
    let first_withdrawal_tx_id = TransactionId::from(2u8);
    let first_withdrawal_amount = Value::from_num(100.0);
    let first_withdrawal = Transaction {
        transaction_type: Withdrawal,
        client_id: Default::default(),
        transaction_id: first_withdrawal_tx_id,
        amount: Some(first_withdrawal_amount),
    };
    assert!(matches!(
        account.process_transaction(&first_withdrawal),
        Ok(_)
    ));
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount
    );
    assert_eq!(account.held_balance, Value::ZERO);

    // A withdrawal cannot be charged back before being disputed first
    let chargeback = Transaction {
        transaction_type: Chargeback,
        client_id: Default::default(),
        transaction_id: first_withdrawal_tx_id,
        amount: None,
    };
    assert_eq!(
        account.process_transaction(&chargeback).unwrap_err(),
        Error::IllegalMovementStatusTransition {
            from: InForce,
            to: ChargedBack,
        }
    );
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount
    );
    assert_eq!(account.held_balance, Value::ZERO);

    // A deposit cannot be disputed after the monetary value it brought has been withdrawn
    let dispute = Transaction {
        transaction_type: Dispute,
        client_id: Default::default(),
        transaction_id: first_deposit_tx_id,
        amount: None,
    };
    assert_eq!(
        account.process_transaction(&dispute).unwrap_err(),
        Error::DisputeAmountExceedsAvailableBalance {
            disputing: first_deposit_amount,
            available: first_deposit_amount - first_withdrawal_amount,
        }
    );
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount
    );
    assert_eq!(account.held_balance, Value::ZERO);

    // An unlocked account can process a dispute on a withdrawal transaction, and the balances
    // should change accordingly (originally withdrawn amount appears as "held")
    let dispute = Transaction {
        transaction_type: Dispute,
        client_id: Default::default(),
        transaction_id: first_withdrawal_tx_id,
        amount: None,
    };
    assert!(matches!(account.process_transaction(&dispute), Ok(_)));
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount
    );
    assert_eq!(account.held_balance, first_withdrawal_amount);

    // An unlocked account can resolve a dispute on a withdrawal transaction, and the balances
    // should change accordingly (the withdrawal is applied, and the held balance disappears)
    let resolve = Transaction {
        transaction_type: Resolve,
        client_id: Default::default(),
        transaction_id: first_withdrawal_tx_id,
        amount: None,
    };
    assert!(matches!(account.process_transaction(&resolve), Ok(_)));
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount
    );
    assert_eq!(account.held_balance, Value::ZERO);

    // An unlocked account can process a dispute on a deposit transaction, and the balances
    // should change accordingly (available balance becomes "held")
    let second_deposit_tx_id = TransactionId::from(3u8);
    let second_deposit_amount = Value::from_num(12345.6789);
    let second_deposit = Transaction {
        transaction_type: Deposit,
        client_id: Default::default(),
        transaction_id: second_deposit_tx_id,
        amount: Some(second_deposit_amount),
    };
    assert!(matches!(
        account.process_transaction(&second_deposit),
        Ok(_)
    ));
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount + second_deposit_amount
    );
    assert_eq!(account.held_balance, Value::ZERO);
    let dispute = Transaction {
        transaction_type: Dispute,
        client_id: Default::default(),
        transaction_id: second_deposit_tx_id,
        amount: None,
    };
    assert!(matches!(account.process_transaction(&dispute), Ok(_)));
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount
    );
    assert_eq!(account.held_balance, second_deposit_amount);

    // An unlocked account can resolve a dispute on a deposit transaction, and the balances
    // should change accordingly (the deposit is applied, and the held balance disappears)
    let resolve = Transaction {
        transaction_type: Resolve,
        client_id: Default::default(),
        transaction_id: second_deposit_tx_id,
        amount: None,
    };
    assert!(matches!(account.process_transaction(&resolve), Ok(_)));
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount + second_deposit_amount
    );
    assert_eq!(account.held_balance, Value::ZERO);

    // An unlocked account can charge back a dispute on a deposit transaction, and the balances
    // should change accordingly (the deposit is reverted, and the held balance disappears)
    let dispute = Transaction {
        transaction_type: Dispute,
        client_id: Default::default(),
        transaction_id: second_deposit_tx_id,
        amount: None,
    };
    assert!(matches!(account.process_transaction(&dispute), Ok(_)));
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount
    );
    assert_eq!(account.held_balance, second_deposit_amount);
    let chargeback = Transaction {
        transaction_type: Chargeback,
        client_id: Default::default(),
        transaction_id: second_deposit_tx_id,
        amount: None,
    };
    assert!(matches!(account.process_transaction(&chargeback), Ok(_)));
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount
    );
    assert_eq!(account.held_balance, Value::ZERO);

    // A charged back transaction cannot be disputed, not only because it is an illegal transition
    // but also because the account will be locked
    let dispute = Transaction {
        transaction_type: Dispute,
        client_id: Default::default(),
        transaction_id: second_deposit_tx_id,
        amount: None,
    };
    assert_eq!(
        account.process_transaction(&dispute).unwrap_err(),
        Error::LockedAccount,
    );

    // A charged back transaction cannot be resolved, not only because it is an illegal transition
    // but also because the account will be locked
    let resolve = Transaction {
        transaction_type: Resolve,
        client_id: Default::default(),
        transaction_id: second_deposit_tx_id,
        amount: None,
    };
    assert_eq!(
        account.process_transaction(&resolve).unwrap_err(),
        Error::LockedAccount,
    );

    // A charged back transaction cannot be charged back again, not only because it is an illegal
    // transition
    let chargeback = Transaction {
        transaction_type: Resolve,
        client_id: Default::default(),
        transaction_id: second_deposit_tx_id,
        amount: None,
    };
    assert_eq!(
        account.process_transaction(&chargeback).unwrap_err(),
        Error::LockedAccount,
    );

    // A locked account admits no further deposits
    let third_deposit_tx_id = TransactionId::from(4u8);
    let third_deposit_amount = Value::from_num(0.1234567);
    let third_deposit = Transaction {
        transaction_type: Deposit,
        client_id: Default::default(),
        transaction_id: third_deposit_tx_id,
        amount: Some(third_deposit_amount),
    };
    assert_eq!(
        account.process_transaction(&third_deposit).unwrap_err(),
        Error::LockedAccount,
    );

    // A locked account admits no further withdrawals
    let second_withdrawal_tx_id = TransactionId::from(4u8);
    let second_withdrawal_amount = Value::from_num(0.1234567);
    let second_withdrawal = Transaction {
        transaction_type: Withdrawal,
        client_id: Default::default(),
        transaction_id: second_withdrawal_tx_id,
        amount: Some(second_withdrawal_amount),
    };
    assert_eq!(
        account.process_transaction(&second_withdrawal).unwrap_err(),
        Error::LockedAccount,
    );

    // Let's unlock the account just for the sake of one last chargeback test.
    // This will be our little secret... 🙊
    account.unlock();

    // An unlocked account can charge back a dispute on a withdrawal transaction, and the balances
    // should change accordingly (the withdrawal is reverted, and the held balance is transferred
    // back to the available balance)
    let dispute = Transaction {
        transaction_type: Dispute,
        client_id: Default::default(),
        transaction_id: first_withdrawal_tx_id,
        amount: None,
    };
    assert!(matches!(account.process_transaction(&dispute), Ok(_)));
    assert_eq!(
        account.available_balance,
        first_deposit_amount - first_withdrawal_amount
    );
    assert_eq!(account.held_balance, first_withdrawal_amount);
    let chargeback = Transaction {
        transaction_type: Chargeback,
        client_id: Default::default(),
        transaction_id: first_withdrawal_tx_id,
        amount: None,
    };
    assert!(matches!(account.process_transaction(&chargeback), Ok(_)));
    assert_eq!(account.available_balance, first_deposit_amount);
    assert_eq!(account.held_balance, Value::ZERO);
}
