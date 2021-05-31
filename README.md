## Running
cargo run -- in.csv > out.csv

## Note on memory usage
Transactions are processed one at a time, but all accounts and transactions are kept in memory after being processed.
Accounts must be kept in memory since stdout is not seekable, so there is nothing that can be done about that.
It's definitely possible to seek back through the input CSV so that records are read from there when a dispute occurs,
but this wasn't implemented.

## Currency
Currency is stored as an integer in "cents" instead of "dollars", or rather in 1/10000s of a currency unit instead of a
currency unit.  This is done to allow easy math on the values and not losing precision. I chose i64 to store the
value, which somewhat limits how much an account can hold, but should work for most currencies in use. This can be
bumped up to i128.
f64 and f32 are generally not good candidates to store exact monetary values, since they can lose precision, and it
would be alarming if adding 1 dollar to an account was not visible, even if you're a billionaire. It also makes
comparing values complicated, since you'd need to always compare with a range instead, since the number is almost never
exactly represented.

## Transactions
### Deposit & Withdraw
These were fairly easy to understand.

### Dispute, Resolve, Chargeback
These operations are slightly weird and only make sense if the party issuing these are a payment processor, but then
withdrawals would have to be handled as well somehow, but there is no requirement for that.

It's also peculiar that they don't have a unique id, since it would help keeping them as events in the history for
auditing reasons.

### Other notes
It's possible to go into a negative total with an account:
deposit, 1, 1, 2
withdraw, 1, 2, 1,
dispute, 1, 1
chargeback, 1, 1