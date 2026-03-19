use crate::{engine::Engine, types::ClientId};

const SAMPLE_INPUT_VECTOR: &str = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0"#;

const SAMPLE_OUTPUT_VECTOR: &str = r#"client,available,held,total,locked
2,2,0,2,false
1,1.5,0,1.5,false
"#;

const COMPLEX_INPUT_VECTOR: &str = r#"type, client, tx, amount
deposit, 1, 1, 1234.5678
deposit, 1, 2, 1111.1111
withdrawal, 1, 3, 1111.1111
resolve, 1, 2,
dispute, 1, 2,
resolve, 1, 2,
dispute, 1, 2,
dispute, 1, 3,
dispute, 1, 3,
resolve, 1, 3,
resolve, 1, 3,
dispute, 1, 3,
chargeback, 1, 3,
chargeback, 1, 3,
"#;

const COMPLEX_OUTPUT_VECTOR: &str = r#"client,available,held,total,locked
1,1234.5678,1111.1111,2345.6789,true
"#;

#[test]
fn sample_csv_loading() {
    let data = SAMPLE_INPUT_VECTOR.as_bytes();

    // Can load transactions from CSV data
    let mut engine = Engine::default();
    assert_eq!(engine.load_transactions_from_reader(data), Ok(()));

    // The total balance of client #1 should be 1.5 (1 + 2 - 1.5 = 1.5)
    assert_eq!(
        engine
            .accounts
            .get_account(ClientId::from(1u8))
            .unwrap()
            .total_balance(),
        1.5
    );

    // The total balance of client #2 should be 2 (2 - 3 = 2)
    // Note that the -3 withdrawal will fail, hence why the balance is still 2
    assert_eq!(
        engine
            .accounts
            .get_account(ClientId::from(2u8))
            .unwrap()
            .total_balance(),
        2.0
    );
}

#[test]
fn sample_csv_outputting() {
    let data = SAMPLE_INPUT_VECTOR.as_bytes();
    let mut engine = Engine::default();
    _ = engine.load_transactions_from_reader(data);

    let mut output = Vec::new();
    _ = engine.output_accounts_into_csv_writer(&mut output);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!(output_str, SAMPLE_OUTPUT_VECTOR);
}

#[test]
fn complex_e2e() {
    let data = COMPLEX_INPUT_VECTOR.as_bytes();
    let mut engine = Engine::default();
    _ = engine.load_transactions_from_reader(data);

    let mut output = Vec::new();
    _ = engine.output_accounts_into_csv_writer(&mut output);
    let output_str = String::from_utf8(output).unwrap();

    assert_eq!(output_str, COMPLEX_OUTPUT_VECTOR);
}
