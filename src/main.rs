use transactor::{cli, engine};

/// The main runtime.
// I tried to keep this as compact as possible by offloading most of the business logic into the CLI
// and Engine components.
fn main() {
    // Initialize logger before anything else.
    env_logger::init();

    // Initialize the main CLI of our application. In essence, it simply tries to parses one and
    // only one argument, which will be used as the path for a CSV file containing transactions.
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
