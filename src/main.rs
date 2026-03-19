/// Everything related to user accounts, balances, etc.
pub mod accounts;
/// Barebones implementation of a CLI that reads a file path from one and only one argument.
pub mod cli;
/// The main data structure that will act as the core of our runtime and will implement the business
/// logic of transaction processing.
pub mod engine;
/// Centralizes error definition and handling.
pub mod errors;
pub mod movements;
/// Everything related with transactions and their semantics.
pub mod transactions;
/// Centralized type definitions for essential data types such as transaction IDS, client IDs,
/// monetary values, etc.; for ease of adjusting them in the future, should the requirements
/// change.
pub mod types;

/// The main runtime.
// I tried to keep this as compact as possible by offloading most of the business logic into the CLI
// and Engine components.
fn main() {
    // Initialize logger before anything else.
    env_logger::init();

    // Initialize the main CLI of our program. In essence, it simply tries to parses one and only
    // one argument, which will be used as the path for a CSV file containing transactions.
    let main_cli = cli::CLI::start();

    // Initialize the main engine that encapsulates the business logic of accounting and processing
    // transactions.
    let mut engine = engine::Engine::default();

    // Load transactions into the engine from the CSV file path provided through the CLI argument.
    if let Err(err) = engine.load_transactions_from_csv_file(&main_cli.csv_file_path) {
        log::error!("{}", err)
    }

    // Print the final state of the accounts into `stout`.
    if let Err(err) = engine.output_accounts_into_stdout() {
        log::error!("{}", err)
    }
}
