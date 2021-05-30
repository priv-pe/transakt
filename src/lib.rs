mod account;
mod currency;
mod transaction;

use crate::transaction::{ClientId, Transaction, TransactionId, TransactionRow};

use crate::account::Account;
use crate::Error::TransactionParseError;
use csv::Trim;
use std::collections::HashMap;
use std::convert::TryInto;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    TransactionParseError,
    DuplicateTransaction(TransactionId),
}

pub struct Transakt {
    accounts: HashMap<ClientId, Account>,
    transactions: HashMap<TransactionId, Transaction>,
}

impl Default for Transakt {
    fn default() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }
}

impl Transakt {
    pub fn read_from_csv(filepath: &Path) -> Result<Transakt, Error> {
        let mut transakt = Self::default();
        let mut csv = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(Trim::All)
            .from_path(filepath)
            .expect("Cannot open input file");
        for record in csv.deserialize() {
            let transaction: TransactionRow = record.map_err(|_| Error::TransactionParseError)?;
            let transaction: Transaction = transaction.try_into()?;
            log::info!("{:?}", transaction);
        }
        Ok(transakt)
    }

    pub fn execute_transaction(&mut self, transaction: Transaction) -> Result<(), Error> {
        match transaction {
            Transaction::Deposit { client, tx, amount } => {
                if self.transactions.contains_key(&tx) {
                    return Err(Error::DuplicateTransaction(tx));
                }
                self.transactions.insert(tx, transaction);
                let account = self.accounts.entry(client).or_insert(Account::new(client));
            }
            _ => todo!(),
        }
        Ok(())
    }
}
