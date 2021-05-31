mod account;
mod currency;
mod transaction;

use crate::transaction::{ClientId, Transaction, TransactionId, TransactionRow};

use crate::account::Account;
use csv::Trim;
use std::collections::HashMap;
use std::convert::TryInto;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    // Big Error
    TransactionParseError,
    InsufficientHeldFunds,

    // Can ignore
    DuplicateTransaction(TransactionId),
    Overflow,
    AccountLocked,
    InsufficientFunds,
    InvalidTransaction,
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
            transakt.execute_transaction(transaction)?;
        }
        Ok(transakt)
    }

    pub fn execute_transaction(&mut self, transaction: Transaction) -> Result<(), Error> {
        match transaction {
            Transaction::Deposit {
                client,
                tx,
                amount,
                disputed,
            } => {
                if amount.is_negative() {
                    log::warn!("Negative withdraw {:?} {:?}", tx, amount);
                    return Err(Error::InvalidTransaction);
                }
                if self.transactions.contains_key(&tx) {
                    log::warn!("Duplicate transaction {:?}", tx);
                    return Err(Error::DuplicateTransaction(tx));
                }
                let account = self.accounts.entry(client).or_insert(Account::new(client));
                //
                account.deposit(amount)?;
                self.transactions.insert(tx, transaction);
            }
            Transaction::Withdrawal { client, tx, amount } => {
                if amount.is_negative() {
                    log::warn!("Negative withdraw {:?} {:?}", tx, amount);
                    return Err(Error::InvalidTransaction);
                }
                if self.transactions.contains_key(&tx) {
                    log::warn!("Duplicate transaction {:?}", tx);
                    return Err(Error::DuplicateTransaction(tx));
                }
                let account = self.accounts.entry(client).or_insert(Account::new(client));
                account.withdraw(amount)?;
                self.transactions.insert(tx, transaction);
            }
            Transaction::Dispute { client, tx } => {
                if let Some(transaction) = self.transactions.get_mut(&tx) {
                    match transaction {
                        Transaction::Deposit {
                            client: client,
                            tx: tx,
                            amount,
                            disputed,
                        } => {
                            if *disputed {
                                log::warn!("Dispute twice on {:?}", tx);
                                return Err(Error::InvalidTransaction);
                            }
                            *disputed = true;
                            // should never happen since we already have an existing transaction.
                            let account = self.accounts.get_mut(client).unwrap();
                            account.hold(*amount);
                        }
                        _ => {
                            log::warn!("Invalid dispute on {:?}", tx);
                        }
                    }
                }
            }
            Transaction::Resolve { client, tx } => {
                if let Some(transaction) = self.transactions.get_mut(&tx) {
                    match transaction {
                        Transaction::Deposit {
                            client: client,
                            tx: tx,
                            amount,
                            disputed,
                        } => {
                            if !*disputed {
                                log::warn!("No dispute on {:?}", tx);
                                return Err(Error::InvalidTransaction);
                            }
                            *disputed = false;
                            // should never happen since we already have an existing transaction.
                            let account = self.accounts.get_mut(client).unwrap();
                            account.release(*amount);
                        }
                        _ => {
                            log::warn!("Invalid dispute on {:?}", tx);
                        }
                    }
                }
            }
            Transaction::Chargeback { tx, .. } => {
                if let Some(transaction) = self.transactions.get_mut(&tx) {
                    match transaction {
                        Transaction::Deposit {
                            client,
                            tx,
                            amount,
                            disputed,
                        } => {
                            if !*disputed {
                                log::warn!("No dispute on {:?}", tx);
                                return Err(Error::InvalidTransaction);
                            }
                            *disputed = false;
                            // should never happen since we already have an existing transaction.
                            let account = self.accounts.get_mut(client).unwrap();
                            account.chargeback(*amount)?;
                        }
                        _ => {
                            log::warn!("Invalid dispute on {:?}", tx);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::currency::Currency;
    use crate::transaction::{ClientId, Transaction, TransactionId};
    use crate::Transakt;

    #[test]
    fn execute_deposit() {
        let mut transakt = Transakt::default();
        // deposit 1.0 into account 1
        transakt
            .execute_transaction(Transaction::Deposit {
                client: ClientId::new(1),
                tx: TransactionId::new(1),
                amount: Currency::new(1, 0).unwrap(),
                disputed: false,
            })
            .unwrap();
        // account 1 shhould have 1.0
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(1, 0).unwrap());
        // deposit 1.0 into account 1
        transakt
            .execute_transaction(Transaction::Deposit {
                client: ClientId::new(1),
                tx: TransactionId::new(2),
                amount: Currency::new(1, 0).unwrap(),
                disputed: false,
            })
            .unwrap();
        // account 1 shhould have 2.0
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(2, 0).unwrap());
        // deposit 0.1 into account 2
        transakt
            .execute_transaction(Transaction::Deposit {
                client: ClientId::new(2),
                tx: TransactionId::new(3),
                amount: Currency::new(0, 1000).unwrap(),
                disputed: false,
            })
            .unwrap();
        // account 1 should have 1, account 2 should have 0.1
        assert_eq!(transakt.accounts.len(), 2);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(2, 0).unwrap());
        let account = transakt.accounts.get(&ClientId::new(2)).unwrap();
        assert_eq!(account.available(), &Currency::new(0, 1000).unwrap());
    }

    #[test]
    fn execute_withdraw() {
        // fund account 1 with 2.0
        let mut transakt = Transakt::default();
        transakt
            .execute_transaction(Transaction::Deposit {
                client: ClientId::new(1),
                tx: TransactionId::new(1),
                amount: Currency::new(2, 0).unwrap(),
                disputed: false,
            })
            .unwrap();
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        // withdraw from account 1 1.0
        assert_eq!(account.available(), &Currency::new(2, 0).unwrap());
        transakt
            .execute_transaction(Transaction::Withdrawal {
                client: ClientId::new(1),
                tx: TransactionId::new(2),
                amount: Currency::new(1, 0).unwrap(),
            })
            .unwrap();
        // account 1 should have 1.0
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(1, 0).unwrap());
        // withdraw from account 1 0.05
        transakt
            .execute_transaction(Transaction::Withdrawal {
                client: ClientId::new(1),
                tx: TransactionId::new(3),
                amount: Currency::new(0, 500).unwrap(),
            })
            .unwrap();
        // account 1 should have 0.95
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(0, 9500).unwrap());
    }

    #[test]
    fn execute_dispute() {
        // fund account 1 with 2.0
        let mut transakt = Transakt::default();
        transakt
            .execute_transaction(Transaction::Deposit {
                client: ClientId::new(1),
                tx: TransactionId::new(1),
                amount: Currency::new(2, 0).unwrap(),
                disputed: false,
            })
            .unwrap();
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        // withdraw from account 1 1.0
        assert_eq!(account.available(), &Currency::new(2, 0).unwrap());
        transakt
            .execute_transaction(Transaction::Dispute {
                client: ClientId::new(1),
                tx: TransactionId::new(1),
            })
            .unwrap();
        // account 1 should have 1.0
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(0, 0).unwrap());
        assert_eq!(account.held(), &Currency::new(2, 0).unwrap());
        assert_eq!(account.total(), Currency::new(2, 0).ok());
        // try withdraw from account 1 0.05
        transakt
            .execute_transaction(Transaction::Withdrawal {
                client: ClientId::new(1),
                tx: TransactionId::new(2),
                amount: Currency::new(0, 500).unwrap(),
            })
            .unwrap_err();
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(0, 0).unwrap());
        assert_eq!(account.held(), &Currency::new(2, 0).unwrap());
        assert_eq!(account.total(), Currency::new(2, 0).ok());
    }

    #[test]
    fn execute_resolve() {
        // fund account 1 with 2.0
        let mut transakt = Transakt::default();
        transakt
            .execute_transaction(Transaction::Deposit {
                client: ClientId::new(1),
                tx: TransactionId::new(1),
                amount: Currency::new(2, 0).unwrap(),
                disputed: false,
            })
            .unwrap();
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        // withdraw from account 1 1.0
        assert_eq!(account.available(), &Currency::new(2, 0).unwrap());
        transakt
            .execute_transaction(Transaction::Dispute {
                client: ClientId::new(1),
                tx: TransactionId::new(1),
            })
            .unwrap();
        // account 1 should have 1.0
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(0, 0).unwrap());
        assert_eq!(account.held(), &Currency::new(2, 0).unwrap());
        assert_eq!(account.total(), Currency::new(2, 0).ok());
        // try withdraw from account 1 0.05
        transakt
            .execute_transaction(Transaction::Resolve {
                client: ClientId::new(1),
                tx: TransactionId::new(1),
            })
            .unwrap();
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(2, 0).unwrap());
        assert_eq!(account.held(), &Currency::new(0, 0).unwrap());
        assert_eq!(account.total(), Currency::new(2, 0).ok());
    }

    #[test]
    fn execute_chargeback() {
        // fund account 1 with 2.0
        let mut transakt = Transakt::default();
        transakt
            .execute_transaction(Transaction::Deposit {
                client: ClientId::new(1),
                tx: TransactionId::new(1),
                amount: Currency::new(2, 0).unwrap(),
                disputed: false,
            })
            .unwrap();
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        // withdraw from account 1 1.0
        assert_eq!(account.available(), &Currency::new(2, 0).unwrap());
        transakt
            .execute_transaction(Transaction::Dispute {
                client: ClientId::new(1),
                tx: TransactionId::new(1),
            })
            .unwrap();
        // account 1 should have 1.0
        assert_eq!(transakt.accounts.len(), 1);
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(0, 0).unwrap());
        assert_eq!(account.held(), &Currency::new(2, 0).unwrap());
        assert_eq!(account.total(), Currency::new(2, 0).ok());
        // try withdraw from account 1 0.05
        transakt
            .execute_transaction(Transaction::Chargeback {
                client: ClientId::new(1),
                tx: TransactionId::new(1),
            })
            .unwrap();
        let account = transakt.accounts.get(&ClientId::new(1)).unwrap();
        assert_eq!(account.available(), &Currency::new(0, 0).unwrap());
        assert_eq!(account.held(), &Currency::new(0, 0).unwrap());
        assert_eq!(account.total(), Currency::new(0, 0).ok());
        assert!(account.is_locked());
    }
}
