use super::*;
use crate::errors::Error::IllegalMovementStatusTransition;

#[test]
fn movement_statuses() {
    use MovementStatus::*;

    let mut movement = Movement::new(
        MovementType::Deposit,
        Value::from_num(123.456),
        Value::from_num(123.456),
    );

    // Initial status is in force
    assert_eq!(movement.status, InForce);

    // Cannot charge back a movement without disputing it first, so status is still in force
    assert_eq!(
        movement.update_status(ChargedBack),
        Err(IllegalMovementStatusTransition {
            from: InForce,
            to: ChargedBack
        })
    );
    assert_eq!(movement.status, InForce);

    // Can dispute a movement that is in force, status is updated accordingly
    assert_eq!(movement.update_status(Disputed), Ok(()));
    assert_eq!(movement.status, Disputed);

    // Can bring back a movement to being in force through resolution, status is updated accordingly
    assert_eq!(movement.update_status(InForce), Ok(()));
    assert_eq!(movement.status, InForce);

    // Can charge back a movement that is under dispute, status is updated accordingly
    _ = movement.update_status(Disputed);
    assert_eq!(movement.update_status(ChargedBack), Ok(()));
    assert_eq!(movement.status, ChargedBack);

    // Once charged back, there is no way back! Status will not change
    assert_eq!(
        movement.update_status(InForce),
        Err(IllegalMovementStatusTransition {
            from: ChargedBack,
            to: InForce
        })
    );
    assert_eq!(movement.status, ChargedBack);
    assert_eq!(
        movement.update_status(Disputed),
        Err(IllegalMovementStatusTransition {
            from: ChargedBack,
            to: Disputed
        })
    );
    assert_eq!(movement.status, ChargedBack);

    // Transitioning from one status to the very same status is useless, so it is forbidden too
    let mut movement = Movement::new(
        MovementType::Deposit,
        Value::from_num(123.456),
        Value::from_num(123.456),
    );
    let statuses = [InForce, Disputed, ChargedBack];
    for status in statuses.into_iter() {
        _ = movement.update_status(status);
        assert_eq!(
            movement.update_status(status),
            Err(IllegalMovementStatusTransition {
                from: status,
                to: status
            })
        );
        assert_eq!(movement.status, status)
    }
}

#[test]
fn balance_history() {
    let mut history = BalanceHistory::default();

    // The history starts empty
    assert!(history.is_empty());
    assert_eq!(history.len(), 0);

    // Once an element is pushed, it can be found in the history and queried by ID
    let first_id = TransactionId::from(1u8);
    let first_movement = Movement::new(
        MovementType::Deposit,
        Value::from_num(123.456),
        Value::from_num(123.456),
    );
    history.push(first_id, first_movement);
    assert!(!history.is_empty());
    assert_eq!(history.len(), 1);
    assert_eq!(history.to_vec()[0], (&first_id, &first_movement));
    assert_eq!(history.get(&first_id), Some(&first_movement));

    // Further elements can be pushed, found in the history and queried by ID
    let second_id = TransactionId::from(2u8);
    let second_movement = Movement::new(
        MovementType::Deposit,
        Value::from_num(123.456),
        Value::from_num(123.456),
    );
    history.push(second_id, second_movement);
    assert!(!history.is_empty());
    assert_eq!(history.len(), 2);
    assert_eq!(history.to_vec()[0], (&first_id, &first_movement));
    assert_eq!(history.to_vec()[1], (&second_id, &second_movement));
    assert_eq!(history.get(&second_id), Some(&second_movement));
}
