/// Everything related to user accounts, balances, etc.
pub mod accounts;
/// The main data structure that will act as the core of our runtime and will implement the business
/// logic of transaction processing.
pub mod engine;
/// Everything related with transactions and their semantics.
pub mod transactions;

fn main() {
    println!("Hello, world!");
}
