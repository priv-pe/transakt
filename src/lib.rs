mod transaction;
mod currency;

use crate::transaction::{Transaction, TransactionRow};

use csv::Trim;
use std::convert::TryInto;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    TransactionParseError,
}

pub fn run(filepath: &Path) -> Result<(), Error> {
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

    Ok(())
}
