use crate::Error;
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct ClientId {
    id: u16,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct TransactionId {
    id: u32,
}

// TODO: currency should be represented as a fixed point value, not as floating point, since
// floating point is not an accurate, lossless representation of a number in all cases.
#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Currency {
    amount: f64,
}

/// Represents a transaction.
#[derive(Debug)]
pub enum Transaction {
    Deposit {
        client: ClientId,
        tx: TransactionId,
        amount: Currency,
    },
    Withdrawal {
        client: ClientId,
        tx: TransactionId,
        amount: Currency,
    },
    Dispute {
        client: ClientId,
        tx: TransactionId,
    },
    Resolve {
        client: ClientId,
        tx: TransactionId,
    },
    Chargeback {
        client: ClientId,
        tx: TransactionId,
    },
}

/// This is a helper type that allows CSV deserialization since CSVs can't deserialize into a
/// typed enum directly
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct TransactionRow {
    #[serde(rename = "type")]
    tx_type: TransactionType,
    client: ClientId,
    tx: TransactionId,
    amount: Option<Currency>,
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = Error;

    fn try_from(t: TransactionRow) -> Result<Transaction, Error> {
        match t {
            TransactionRow {
                tx_type: TransactionType::Deposit,
                client,
                tx,
                amount: Some(amount),
            } => Ok(Transaction::Deposit { client, tx, amount }),
            TransactionRow {
                tx_type: TransactionType::Withdrawal,
                client,
                tx,
                amount: Some(amount),
            } => Ok(Transaction::Withdrawal { client, tx, amount }),
            TransactionRow {
                tx_type: TransactionType::Dispute,
                client,
                tx,
                amount: None,
            } => Ok(Transaction::Dispute { client, tx }),
            TransactionRow {
                tx_type: TransactionType::Resolve,
                client,
                tx,
                amount: None,
            } => Ok(Transaction::Resolve { client, tx }),
            TransactionRow {
                tx_type: TransactionType::Chargeback,
                client,
                tx,
                amount: None,
            } => Ok(Transaction::Chargeback { client, tx }),
            _ => Err(Error::TransactionParseError),
        }
    }
}
