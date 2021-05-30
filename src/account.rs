use crate::transaction::ClientId;
use crate::currency::Currency;

pub struct Account {
    client: ClientId,
    available: Currency,
    held: Currency,
    locked: bool,
}

impl Account {
    pub fn new(client: ClientId) -> Account {
        Self {
            client,
            available: Currency::default(),
            held: Currency::default(),
            locked: false,
        }
    }
}