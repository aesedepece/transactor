use crate::{
    accounts::{Account, AccountsSystem},
    errors::Error,
    transactions::Transaction,
};
use std::fs::File;

#[cfg(test)]
mod tests;

/// The main component that conforms the runtime of this app:
/// - In its very core lays one instance of an accounts system.
/// - Provides functions for loading and decoding transactions from CSV files, as well as for
///   representing and outputting the final state of the accounts in CSV format.
pub struct TransactorEngine {
    /// The accounts system that will track balances and movements, and will be able to process
    /// transactions.
    accounts: AccountsSystem,
}

impl TransactorEngine {
    /// Apply a transaction on an account allegedly contained in the accounts system, internally
    /// mutating it as expected from the semantics of the transaction type.
    ///
    /// Upon success, returns the final state of the account, i.e. how it looks like after mutation.
    pub fn process_transaction(&mut self, transaction: &Transaction) -> Result<&Account, Error> {
        self.accounts.process_transaction(transaction)
    }

    /// Load, decode and process transactions from a generic read handle that allegedly contains CSV
    /// data representing transactions.
    ///
    /// The use of the generics and static dispatching here allows us to use this function both for
    /// reading from a local file in the actual runtime, and from a data structure in tests.
    pub fn load_transactions_from_reader<R>(&mut self, reader: R) -> Result<(), Error>
    where
        R: std::io::Read,
    {
        // Build a buffered CSV reader around a generic implementor of `std::io::Read`.
        // The CSV file is NOT assumed to have headers because I want to leave the door open to that
        // possibility, and processing the header line will fail gracefully anyway.
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .trim(csv::Trim::All)
            .from_reader(reader);

        // Now we can stream the lines into being processed as transactions
        // Because transaction ordering matters, this is done in an iterative way. Otherwise, this
        // could benefit from parallelization using the likes of `rayon`.
        for line in reader.deserialize::<Transaction>() {
            let transaction = line.map_err(Error::from)?;
            self.process_transaction(&transaction)?;
        }

        Ok(())
    }

    /// Performs the whole CSV file reading, decoding and processing part of this app.
    ///
    /// Most likely called from `main()`.
    pub fn load_transactions_from_csv_file(&mut self, path: &str) -> Result<(), Error> {
        // Obtain a read handle over the CSV file
        let read = File::open(path).map_err(Error::from)?;
        // Trigger the actual loading of the transactions from the read handle
        self.load_transactions_from_reader(read)
    }
}
