/// Everything related to user accounts, balances, etc.
pub mod accounts;
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

fn main() {
    println!("Hello, world!");
}
