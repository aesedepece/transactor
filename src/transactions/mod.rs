use crate::types::*;
use serde::Deserialize;
#[cfg(test)]
mod tests;

/// Covers the different types of transactions that we can apply on an user account.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    /// A user deposited an amount of value that needs to be added to their balance.
    Deposit,
    /// A user wants to withdraw an amount of value that needs to be removed from their balance.
    Withdrawal,
    /// A user claimed that a transaction was erroneous and should be reversed. The amount added or
    /// removed by the disputed transaction should be added to their "held" balance.
    Dispute,
    /// A dispute has been rejected, resulting in the disputed transaction taking full force again,
    /// and held balances being released.
    ///
    /// That is, a disputed deposit that ends up with a resolution will eventually result in held
    /// balance becoming available again; while a disputed withdrawal that ends up with a resolution
    /// will result in the withdrawn balance becoming available again.
    Resolve,
    /// A dispute has been accepted, resulting in full reversal of the disputed transaction's
    /// semantics.
    ///
    /// That is, a disputed deposit that ends up with a chargeback will eventually result in no
    /// balance being added; while a disputed withdrawal that ends up with a chargeback
    ///
    /// Chargebacks also place the user's account into "frozen" state.
    Chargeback,
}

/// The main data structure holding data for a transaction.
#[derive(Debug, Deserialize)]
pub struct Transaction {
    /// Tells how to process the transaction based on what it is representing, e.g. (deposits,
    /// withdrawals, etc.)
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    /// The unique ID of the client that initiated this transaction; or the client that initiated
    /// the transaction referred in `transaction_id` in the case of disputes, resolutions, and
    /// chargebacks.
    #[serde(rename = "client")]
    pub client_id: ClientId,
    /// The unique ID of this transaction; or he unique ID of a `deposit` or `withdraw` transaction
    /// in the case of disputes, resolutions, and chargebacks.
    #[serde(rename = "tx")]
    pub transaction_id: TransactionId,
    /// The amount of value being deposited or withdrawn.
    ///
    /// This field is optional because only deposits and withdrawals carry an amount. The amount in
    /// question for any other transaction type must be obtained from the original transaction being
    /// disputed, resolved or charged back.
    pub amount: Option<Value>,
}
