use serde::{de::Visitor, Deserialize};
use serde_json::Value;
use starknet_crypto::Felt;
use std::{ops::Deref, str::FromStr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VecFeltError {
    #[error(transparent)]
    NumberParseError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    BigIntParseError(#[from] num_bigint::ParseBigIntError),
    #[error("number out of range")]
    NumberOutOfRange,
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
}

/// `VecFelt` is a wrapper around a vector of `Arg`.
///
/// It provides convenience methods for working with a vector of `Arg` and implements
/// `Deref` to allow it to be treated like a vector of `Arg`.
#[derive(Debug, Clone)]
pub struct VecFelt(Vec<Felt>);

impl VecFelt {
    /// Creates a new `VecFelt` from a vector of `Arg`.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `Arg`.
    ///
    /// # Returns
    ///
    /// * `VecFelt` - A new `VecFelt` instance.
    #[must_use]
    pub fn new(args: Vec<Felt>) -> Self {
        Self(args)
    }
}

impl IntoIterator for VecFelt {
    type Item = Felt;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for VecFelt {
    type Target = Vec<Felt>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<VecFelt> for Vec<Felt> {
    fn from(args: VecFelt) -> Self {
        args.0
    }
}

impl From<Vec<Felt>> for VecFelt {
    fn from(args: Vec<Felt>) -> Self {
        Self(args)
    }
}

impl VecFelt {
    fn visit_seq_helper(seq: &[Value]) -> Result<Self, VecFeltError> {
        let iterator = seq.iter();
        let mut args = Vec::new();

        for arg in iterator {
            match arg {
                Value::Number(n) => {
                    let n = n.as_u64().ok_or(VecFeltError::NumberOutOfRange)?;
                    args.push(Felt::from(n));
                }
                Value::String(n) => {
                    let n = num_bigint::BigUint::from_str(n)?;
                    let mut array = [0u8; 32];
                    let start_index = 32 - n.to_bytes_be().len();
                    array[start_index..].copy_from_slice(&n.to_bytes_be());
                    let felt = Felt::from_bytes_be(&array);
                    args.push(felt);
                }
                Value::Array(a) => {
                    args.push(Felt::from(a.len()));
                    let result = Self::visit_seq_helper(a)?;
                    args.extend(result.0);
                }
                _ => (),
            }
        }

        Ok(Self::new(args))
    }
}

impl<'de> Visitor<'de> for VecFelt {
    type Value = VecFelt;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a list of arguments")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut args = Vec::new();
        while let Some(arg) = seq.next_element()? {
            match arg {
                Value::Number(n) => args.push(Value::Number(n)),
                Value::String(n) => args.push(Value::String(n)),
                Value::Array(a) => args.push(Value::Array(a)),
                _ => return Err(serde::de::Error::custom("Invalid type")),
            }
        }

        Self::visit_seq_helper(&args).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for VecFelt {
    fn deserialize<D>(deserializer: D) -> Result<VecFelt, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(VecFelt(Vec::new()))
    }
}
