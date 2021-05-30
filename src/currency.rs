use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum CurrencyError {
    Overflow,
    DecimalError,
}

#[derive(Debug, PartialEq)]
pub enum CurrencyFormatError {
    InvalidRepresentation,
}

/// Representation of test currency, which holds up to four digits of precision.
/// The upper bound is not specified, but assuming that u64 should be sufficient.
/// In any real system, this would need be more generic, to allow for multiple currencies to exist
/// without implementing a separate structure for each one.
#[derive(Debug, Eq, PartialEq)]
pub struct Currency {
    /// Holds the value as a single integer, without decimals.
    /// holding currency like this is that it's easier to add and multiply without dealing with
    /// complex multiplication logic.
    /// Since we want to represent these values exactly, a f32 or f64 would not have worked for the
    /// purpose.
    amount: u64,
}

impl Default for Currency {
    fn default() -> Self {
        Self { amount: 0 }
    }
}

impl Currency {
    /// How much is one unit in the decimal representation.
    /// Examples:
    ///  * 1USD = 100 cents, DECIMAL_DIGITS = 2
    ///  * 1BTC = 100_000_000 Sats, DECIMAL_DIGITS = 8
    const DECIMAL_DIGITS: u32 = 4;
    const UNIT_IN_DECIMALS: u64 = 10u64.pow(Self::DECIMAL_DIGITS);

    /// Creates a MyCoinValue from a unitary value plus the decimal part.
    pub fn new(unit: u64, decimal: u64) -> Result<Self, CurrencyError> {
        let value = unit
            .checked_mul(Currency::UNIT_IN_DECIMALS)
            .ok_or(CurrencyError::Overflow)?;
        if decimal < Currency::UNIT_IN_DECIMALS {
            // The decimals are in the lower bits and have been reserved, so can't overflow
            Ok(Self {
                amount: value + decimal,
            })
        } else {
            Err(CurrencyError::DecimalError)
        }
    }

    pub fn checked_add(self, other: Self) -> Option<Currency> {
        Some(Currency {
            amount: self.amount.checked_add(other.amount)?,
        })
    }

    pub fn checked_sub(self, other: Self) -> Option<Currency> {
        Some(Currency {
            amount: self.amount.checked_sub(other.amount)?,
        })
    }
}

impl FromStr for Currency {
    type Err = CurrencyFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fields: Vec<&str> = s.split('.').collect();
        match fields.as_slice() {
            [units] => {
                let units = units
                    .parse()
                    .map_err(|_| CurrencyFormatError::InvalidRepresentation)?;
                Currency::new(units, 0).map_err(|_| CurrencyFormatError::InvalidRepresentation)
            }
            [units, decimals] => {
                let units = units
                    .parse()
                    .map_err(|_| CurrencyFormatError::InvalidRepresentation)?;
                let mut decimals: String = if decimals.len() > 0 {
                    decimals.chars().collect()
                } else {
                    "0".to_string()
                };
                // The logic for the decimals is bit more complicated, since the lower end
                // can be eluded, but are important. Simply parsing 0001 and 1 will get us the same
                // result, but we want 0.1 to be 1000 times larger than 0.0001.
                // To deal with this, first ensure that all the characters are digits
                if !decimals.chars().all(|c| c.is_digit(10)) {
                    return Err(CurrencyFormatError::InvalidRepresentation);
                }
                // Then, cut the digits that are not significant.
                decimals.truncate(Currency::DECIMAL_DIGITS as usize);
                // Finally, the number might need to be adjusted, to get the right fraction
                let multiplier = 10u64.pow(Currency::DECIMAL_DIGITS - decimals.len() as u32);
                let decimals = decimals
                    .parse::<u64>()
                    .map_err(|_| CurrencyFormatError::InvalidRepresentation)?;
                let decimals = decimals * multiplier;

                Currency::new(units, decimals)
                    .map_err(|_| CurrencyFormatError::InvalidRepresentation)
            }
            _ => Err(CurrencyFormatError::InvalidRepresentation),
        }
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let units = self.amount / Self::UNIT_IN_DECIMALS;
        let decimals = self.amount % Self::UNIT_IN_DECIMALS;
        write!(f, "{}.{:04}", units, decimals)
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D>(deserializer: D) -> Result<Currency, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Currency::from_str(&s).map_err(|err| D::Error::custom(format!("{:?}", err)))
    }
}

impl Serialize for Currency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

#[cfg(test)]
mod tests {
    use super::Currency;
    use super::CurrencyError;
    use std::str::FromStr;

    #[test]
    fn test_new_ok() {
        let x = Currency::new(2, 1).unwrap();
        assert_eq!(x.amount, 2 * 10000 + 1);
    }

    #[test]
    fn test_new_fail_overflow() {
        let x = Currency::new(10u64.pow(16), 9999).unwrap_err();
        assert_eq!(x, CurrencyError::Overflow);
    }

    #[test]
    fn test_new_fail_decimals() {
        let x = Currency::new(0, 10000).unwrap_err();
        assert_eq!(x, CurrencyError::DecimalError);
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            Currency::from_str("1234").unwrap(),
            Currency::new(1234, 0).unwrap()
        );
        assert_eq!(
            Currency::from_str("1234.1").unwrap(),
            Currency::new(1234, 1000).unwrap()
        );
        assert_eq!(
            Currency::from_str("1234.01").unwrap(),
            Currency::new(1234, 100).unwrap()
        );
        assert_eq!(
            Currency::from_str("1234.").unwrap(),
            Currency::new(1234, 0).unwrap()
        );
        assert_eq!(
            Currency::from_str("01234").unwrap(),
            Currency::new(1234, 0).unwrap()
        );
        assert_eq!(
            Currency::from_str("1234.00000").unwrap(),
            Currency::new(1234, 0).unwrap()
        );
        assert_eq!(
            Currency::from_str("1234.00001").unwrap(),
            Currency::new(1234, 0).unwrap()
        );
        assert_eq!(
            Currency::from_str("1234.00011").unwrap(),
            Currency::new(1234, 1).unwrap()
        );
        assert_eq!(
            Currency::from_str("1234.9876").unwrap(),
            Currency::new(1234, 9876).unwrap()
        );
        assert_eq!(
            Currency::from_str("1234.0806").unwrap(),
            Currency::new(1234, 806).unwrap()
        );
    }

    #[test]
    fn test_from_str_error() {
        Currency::from_str("").unwrap_err();
        Currency::from_str(".").unwrap_err();
        Currency::from_str("123a").unwrap_err();
        Currency::from_str("123a").unwrap_err();
        Currency::from_str("1235.000a").unwrap_err();
        Currency::from_str("1235.0000a").unwrap_err();
        Currency::from_str("1235.0000.00").unwrap_err();
        Currency::from_str("a1235.").unwrap_err();
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Currency::new(1, 0).unwrap()), "1.0000");
        assert_eq!(format!("{}", Currency::new(1, 1).unwrap()), "1.0001");
        assert_eq!(
            format!("{}", Currency::new(1234, 9999).unwrap()),
            "1234.9999"
        );
        assert_eq!(format!("{}", Currency::new(0, 1000).unwrap()), "0.1000");
    }

    #[test]
    fn test_add() {
        let am1 = Currency::new(1, 0).unwrap();
        let am2 = Currency::new(1, 0).unwrap();
        let res = Currency::new(2, 0).unwrap();
        let sum = am1.checked_add(am2).unwrap();
        assert_eq!(sum, res);

        let am1 = Currency::new(0, 100).unwrap();
        let am2 = Currency::new(0, 200).unwrap();
        let res = Currency::new(0, 300).unwrap();
        let sum = am1.checked_add(am2).unwrap();
        assert_eq!(sum, res);

        let am1 = Currency::new(0, 1000).unwrap();
        let am2 = Currency::new(0, 9000).unwrap();
        let res = Currency::new(1, 0).unwrap();
        let sum = am1.checked_add(am2).unwrap();
        assert_eq!(sum, res);
    }

    #[test]
    fn test_sub() {
        let am1 = Currency::new(2, 0).unwrap();
        let am2 = Currency::new(1, 0).unwrap();
        let res = Currency::new(1, 0).unwrap();
        let sum = am1.checked_sub(am2).unwrap();
        assert_eq!(sum, res);

        let am1 = Currency::new(1, 0).unwrap();
        let am2 = Currency::new(0, 1).unwrap();
        let res = Currency::new(0, 9999).unwrap();
        let sum = am1.checked_sub(am2).unwrap();
        assert_eq!(sum, res);

        let am1 = Currency::new(1000, 0).unwrap();
        let am2 = Currency::new(0, 1).unwrap();
        let res = Currency::new(999, 9999).unwrap();
        let sum = am1.checked_sub(am2).unwrap();
        assert_eq!(sum, res);
    }
}
