<div align="center">
    <h1>TRANSACTOR</h1>
    <p><strong>A toy (yet intentionally robust) transaction processing engine and accounting system with support for deposits, withdrawals, disputes, resolutions and chargebacks.</strong></p>
    <br/>
    <a href="https://github.com/aesedepece/transactor/actions"><img src="https://github.com/aesedepece/transactor/actions/workflows/ci.yml/badge.svg" alt="Build Status" /></a>
    <a href="https://github.com/aesedepece/transactor/graphs/contributors"><img src="https://img.shields.io/github/contributors/aesedepece/transactor.svg" alt="GitHub contributors" /></a>
    <a href="https://github.com/aesedepece/transactor/commits/master"><img src="https://img.shields.io/github/last-commit/aesedepece/transactor.svg" alt="Github last commit" /></a>
    <a href="https://github.com/aesedepece/transactor/blob/master/LICENSE"><img src="https://img.shields.io/github/license/aesedepece/transactor.svg" alt="GPLv3 Licensed" /></a>
</div>

## Design Philosophy

* **Streaming I/O:** Designed to handle files larger than available RAM. It processes transactions row-by-row using buffered readers, maintaining a compact memory footprint.
* **Dependency Austerity:** Dependencies are kept as minimal as possible in order to avoid bloating the project and potential security risks.
* **Error Safety:** All business logic errors and I/O failures are logged into `stderr` with configurable log levels, ensuring `stdout` remains a clean data stream for downstream pipes or file redirection.

## Usage

The application accepts a single argument: the path to the input CSV file.

```bash
cargo run -- transactions.csv > accounts.csv
```

As per the requirements, adding any extra arguments will make the application fail.

## Command Line Interface

| Argument                   | Type | Description                                           |
|:---------------------------| :--- | :---------------------------------------------------- |
| `path_to_transactions_csv` | Path | The path to the CSV file containing transaction data. |

### Logging

Logging of errors and diagnostics are handled via the `log` crate abstraction.

To view processing details or debug information without corrupting the CSV output, use the `RUST_LOG` environment variable:

```bash
# View errors only (default)
RUST_LOG=error cargo run -- transactions.csv > accounts.csv

# View detailed processing steps
RUST_LOG=trace cargo run -- transactions.csv > accounts.csv
```

## Assumptions

These are some assumptions that were made on unclear requirements or behaviors that are not clearly defined in the specification:
- Only deposits and withdrawal transactions can be disputed, resolved and charged back.
- Disputing a withdrawal transaction must actually violate the "total funds should remain the same" principle.
- Disputing a deposit should fail if the current balance of the account is less than the disputed amount.
- The `value` field of disputes, resolves and chargebacks should be quietly ignored.
- Input CSV files always contain headers (the first line in a file will be skipped when reading transactions).

## Technical Decisions

Note: additional rationale on specific design decisions and code style choices can be found inline in affected modules.

### Precision & Arithmetic
While the specification hints the use of floating-point values for monetary amounts, in a real production environment, fixed-point decimals or integer-based "bitcoin-like" math is mandatory to avoid unpredictable rounding errors due to lack of precision.

For this implementation, I am leveraging `FixedU64<U14>` from the `fixed` crate to satisfy the required four decimal places of precision.

### Data Structures
Client accounts are stored in an `IndexMap`. This provides $O(1)$ lookup time while maintaining deterministic iteration based on insertion order.

### Testing Determinism
To ensure that test vectors remain reproducible across different environments, the test suite utilizes conditional compilation (`#[cfg(test)]`) to sort account output by `client_id` before writing.

This deterministic behavior can also be enforced in production by enabling the `deterministic` feature flag. 

### Transaction Safety and State Machine
The engine handles the lifecycle of disputes by deriving "movements" from deposit and withdrawal transactions, and storing the movements with the account.
This enables retrieving the original transaction amount upon processing a subsequent dispute, resolve or chargeback 
affecting it.

To ensure financial integrity, the state machine only allows 3 legal transitions:
- **InForce** → **Disputed**, upon processing a dispute on an undisputed deposit or withdrawal transaction.
- **Disputed** → **ChargedBack**, upon processing a chargeback on an already-disputed deposit or withdrawal. 
- **Disputed** → **InForce**, upon processing a resolve on an already-disputed deposit or withdrawal.

Any other transitions are strictly forbidden.

## Testing

The suite includes unit tests for core logic and integration tests for CSV streaming.

```bash
# Run all tests
cargo test
```

To verify the engine against specific test vectors:
```bash
cargo run --features deterministic -- tests/test_input.csv > actual_output.csv
diff actual_output.csv tests/expected_output.csv
```

## Dependencies

### `csv`
Provides the main CSV deserialization and serialization functionality. Suggested by the specification.

### `fixed`
Gives us fixed precision floats, which are strictly needed to guarantee 4-digit decimal precision.
Feature flag `serde-str` is needed for deserialization and serialization from and into CSVs.
This crate was favored for the simplicity of its API, but the `rust_decimal` crate would be equally good here.

### `indexmap`
Clever solution to having a transaction history where you can query by key and get the whole list of entries in the original order of insertion, both with $O(1)$ complexity.

### `thiserror`
The gold standard for error handling and formatting these days, better in every way than the old `failure`.

### `serde`
Needed for deriving deserialization and serialization for transactions. Suggested by the specification.

### `log`
A must for proper logging of runtime errors.

### `env_logger`
Goes hand in hand with `log`.
