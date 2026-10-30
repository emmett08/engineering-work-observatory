//! Parse declared rational quantities without passing through floating point.
use anyhow::{Context, Result, ensure};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Zero;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

/// An exact rational encoded in JSON as an integer or `numerator/denominator` string.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Exact(pub BigRational);

impl FromStr for Exact {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        ensure!(value.len() <= 256, "rational input exceeds 256 characters");
        let parts: Vec<_> = value.split('/').collect();
        ensure!(
            parts.len() == 1 || parts.len() == 2,
            "expected integer or numerator/denominator"
        );
        let integer = |part: &str| -> Result<BigInt> {
            let digits = part.strip_prefix('-').unwrap_or(part);
            ensure!(
                !digits.is_empty() && digits.bytes().all(|c| c.is_ascii_digit()),
                "invalid rational integer"
            );
            BigInt::from_str(part).context("invalid rational integer")
        };
        let numerator = integer(parts[0])?;
        let denominator = if parts.len() == 2 {
            integer(parts[1])?
        } else {
            BigInt::from(1)
        };
        ensure!(
            !denominator.is_zero(),
            "rational denominator must be nonzero"
        );
        Ok(Self(BigRational::new(numerator, denominator)))
    }
}

impl fmt::Display for Exact {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl Serialize for Exact {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Exact {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_integer_precision_and_normalises_fractions() {
        for (input, canonical) in [
            ("9007199254740993", "9007199254740993"),
            ("-6/8", "-3/4"),
            ("3/-6", "-1/2"),
            ("0/13", "0"),
        ] {
            let number: Exact = input.parse().unwrap();
            assert_eq!(number.to_string(), canonical);
            assert_eq!(
                serde_json::from_str::<Exact>(&serde_json::to_string(&number).unwrap()).unwrap(),
                number
            );
        }
    }

    #[test]
    fn rejects_float_and_malformed_inputs() {
        for input in [
            "\"1.5\"",
            "\"1e3\"",
            "1",
            "0.5",
            "null",
            "\"1/0\"",
            "\"1/2/3\"",
            "\" 1\"",
            "\"+1\"",
            "\"\"",
        ] {
            assert!(serde_json::from_str::<Exact>(input).is_err(), "{input}");
        }
        assert!("1".repeat(257).parse::<Exact>().is_err());
    }
}
