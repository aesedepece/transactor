use log;
use std::io::{BufReader, stdin};
use transactor::engine::Engine;

fn main() {
    // Initialize logger before anything else.
    env_logger::init();

    // Initialize our transactions processing engine
    let mut engine = Engine::default();

    // Set up an asynchronous stdin reader
    let stdin = stdin();
    let reader = BufReader::new(stdin);

    // Let the engine process everything from the stdin reader
    engine.load_transactions_from_reader(reader);

    // Upon EOF (or manually, Ctrl+D) write the final account status into stdout
    if let Err(err) = engine.output_accounts_into_stdout() {
        log::error!("{}", err)
    }
}
