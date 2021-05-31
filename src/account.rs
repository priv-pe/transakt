use crate::currency::Currency;
use crate::transaction::ClientId;
use crate::Error;

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

    pub fn available(&self) -> &Currency {
        &self.available
    }

    pub fn held(&self) -> &Currency {
        &self.held
    }

    pub fn total(&self) -> Option<Currency> {
        self.available.checked_add(self.held)
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn deposit(&mut self, amount: Currency) -> Result<(), Error> {
        if !self.is_locked() {
            let sum = self.available.checked_add(amount);
            self.available = sum.ok_or(Error::Overflow)?;
            Ok(())
        } else {
            Err(Error::AccountLocked)
        }
    }

    pub fn withdraw(&mut self, amount: Currency) -> Result<(), Error> {
        if !self.is_locked() {
            let diff = self.available.checked_sub(amount).ok_or(Error::Overflow)?;
            if diff.is_negative() {
                return Err(Error::InsufficientFunds);
            }
            self.available = diff;
            Ok(())
        } else {
            Err(Error::AccountLocked)
        }
    }

    pub fn hold(&mut self, amount: Currency) -> Result<(), Error> {
        if !self.is_locked() {
            // TODO: Should happen atomically
            let sum = self.held.checked_add(amount);
            self.held = sum.ok_or(Error::Overflow)?;
            let diff = self.available.checked_sub(amount);
            self.available = diff.ok_or(Error::Overflow)?;
            Ok(())
        } else {
            Err(Error::AccountLocked)
        }
    }

    pub fn release(&mut self, amount: Currency) -> Result<(), Error> {
        if !self.is_locked() {
            // TODO: Should happen atomically
            let diff = self.held.checked_sub(amount);
            self.held = diff.ok_or(Error::Overflow)?;
            if self.held.is_negative() {
                // This should never happen
                return Err(Error::InsufficientHeldFunds);
            }
            let sum = self.available.checked_add(amount);
            self.available = sum.ok_or(Error::Overflow)?;
            Ok(())
        } else {
            Err(Error::AccountLocked)
        }
    }
}
