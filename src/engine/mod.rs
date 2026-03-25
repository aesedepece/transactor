use crate::{
    accounts::{Account, AccountsSystem},
    errors::Error,
    transactions::Transaction,
};

#[cfg(test)]
mod tests;

/// The main component that conforms the runtime of this application:
/// - In its very core lays one instance of an accounts system.
/// - Provides functions for loading and decoding transactions from CSV files, as well as for
///   representing and outputting the final state of the accounts in CSV format.
#[derive(Default)]
pub struct Engine {
    /// The accounts system that will track balances and movements, and will be able to process
    /// transactions.
    accounts: AccountsSystem,
}

impl Engine {
    /// Apply a transaction on an account allegedly contained in the accounts system, internally
    /// mutating it as expected from the semantics of the transaction type.
    ///
    /// Upon success, returns the final state of the account, i.e. how it looks like after mutation.
    ///
    /// # Errors
    /// Can fail if the account is locked, or if any other circumstance related to the account
    /// or the transaction makes it impossible to process it.
    pub fn process_transaction(&mut self, transaction: &Transaction) -> Result<&Account, Error> {
        self.accounts.process_transaction(transaction)
    }

    /// Load, decode and process transactions from a generic read handle that allegedly contains CSV
    /// data representing transactions.
    ///
    /// The use of generics and static dispatching here allows us to use this function both for
    /// reading from a local file in the actual runtime, and from a data structure in tests.
    ///
    /// Because of its best-effort approach and error forgiveness, this function is infallible in
    /// practice.
    pub fn load_transactions_from_reader<R>(&mut self, reader: R)
    where
        R: std::io::Read,
    {
        // Build a buffered CSV reader around a generic implementor of `std::io::Read`.
        // The CSV file IS assumed to have headers.
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(reader);

        // Now we can stream the lines into being processed as transactions
        // Because transaction ordering matters, this is done in an iterative way. Otherwise, this
        // could benefit from parallelization using the likes of `rayon`.
        // Errors when processing lines of CSV data are expected not to stop the processing, i.e.
        // upon encountering an error, it is simply logged but the CSV data keeps being read.
        for line in reader.deserialize::<Transaction>() {
            _ = line
                // CSV deserialization errors are mapped into our native errors
                .map_err(Error::from)
                // Here we trigger the actual processing of the transaction
                .and_then(|transaction| self.process_transaction(&transaction))
                // Errors are inspected and logged as warnings, but never unwrapped or `?`ed.
                .inspect_err(|err| log::warn!("{}", err));
        }
    }

    /// Performs the whole CSV file reading, decoding and processing part of this application.
    ///
    /// Most likely called from `main()`.
    ///
    /// # Errors
    /// Can fail if the file does not exist, it cannot be opened, or memory allocation fails for the
    /// internal buffer.
    pub fn load_transactions_from_csv_file(&mut self, path: &str) -> Result<(), Error> {
        // Obtain a read handle over the CSV file
        let read = std::fs::File::open(path).map_err(Error::from)?;
        // Trigger the actual loading of the transactions from the read handle
        self.load_transactions_from_reader(read);

        Ok(())
    }

    /// Encode and write account states (namely, account lines) into a generic writer.
    ///
    /// The use of generics and static dispatching here allows us to use this function both for
    /// reading from a local file in the actual runtime, and from a data structure in tests.
    ///
    /// # Errors
    /// Can fail if the output writer cannot be written into or flushed.
    pub fn output_accounts_into_csv_writer<W>(&self, writer: W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        // Build a buffered CSV writer around a generic implementor of `std::io::Write`.
        // The CSV output will have headers.
        let mut csv_writer = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        // Conditional compiling is used here because:
        // - The `HashMap` inside `AccountsSystem` is very efficient, but ordering is
        //   indeterministic.
        // - Tests require deterministic ordering because I want to test against known test vector
        //   strings.
        // - Actual runtime requires no ordering as per the specification, so it would not make
        //   sense to have a performance penalty.
        // As a consequence, the potentially costly ordering only happens if called from tests, or
        // if the `deterministic` feature flag is set.
        let account_lines = self.accounts.get_all_account_lines();
        #[cfg(any(test, feature = "deterministic"))]
        let account_lines = {
            let mut account_lines_vector = account_lines.collect::<Vec<_>>();
            account_lines_vector.sort_by_key(|line| line.client_id);

            account_lines_vector.into_iter().rev()
        };

        // Now we can stream the account lines into being written into the writer.
        // Out of caution, errors when serializing and writing DO stop the processing, i.e.
        // upon encountering an error in one line, the writing operation is aborted.
        for account_line in account_lines {
            csv_writer
                .serialize(account_line)
                .map_err(Error::from)
                .inspect_err(|err| log::error!("{}", err))?;
        }

        // Writers must be flush upon completion of the writing to make sure that data goes to the
        // output regardless of reader backpressure. Otherwise, we could exit the application while
        // the writing is incomplete.
        csv_writer.flush().map_err(Error::from)?;

        Ok(())
    }

    /// Performs the actual CSV-formattin and "printing" of account data into `stdout`.
    ///
    /// Most likely called from `main()`.
    ///
    /// # Errors
    /// Can fail if the `stdout` writer cannot be written into or flushed.
    pub fn output_accounts_into_stdout(&self) -> Result<(), Error> {
        // Obtaining a writer over `stdout` is dead simple
        let writer = std::io::stdout();
        // Offload the heavy lifting to the generic function above
        self.output_accounts_into_csv_writer(writer)
    }
}
