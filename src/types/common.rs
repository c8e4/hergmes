use std::fmt::{Debug, Formatter};

use hex::{FromHex, ToHex};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A 32-byte hash digest.
pub type HashDigest = Digest<32>;

/// A fixed-size byte array represented as a hex string in serialization.
#[derive(Clone, PartialEq, Eq)]
pub struct Digest<const N: usize>(pub [u8; N]);

impl<const N: usize> ToString for Digest<N> {
    fn to_string(&self) -> String {
        self.0.encode_hex::<String>()
    }
}

impl<const N: usize> Into<String> for Digest<N> {
    fn into(self) -> String {
        self.to_string()
    }
}

impl<const N: usize> Debug for Digest<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.encode_hex::<String>())
    }
}

impl<'de, const N: usize> Deserialize<'de> for Digest<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes_vec = Vec::from_hex(&hex_str).map_err(D::Error::custom)?;
        let len = bytes_vec.len();
        let bytes: [u8; N] = bytes_vec
            .try_into()
            .map_err(|_| D::Error::custom(format!("expected {} bytes, got {}", N, len)))?;

        Ok(Digest(bytes))
    }
}

impl<const N: usize> Serialize for Digest<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// A byte vector represented as a hex string in serialization.
#[derive(Clone, PartialEq, Eq)]
pub struct HexBytes(pub Vec<u8>);

impl ToString for HexBytes {
    fn to_string(&self) -> String {
        self.0.encode_hex::<String>()
    }
}

impl Into<String> for HexBytes {
    fn into(self) -> String {
        self.to_string()
    }
}

impl Debug for HexBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.encode_hex::<String>())
    }
}

impl<'de> Deserialize<'de> for HexBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes = Vec::from_hex(&hex_str).map_err(D::Error::custom)?;
        Ok(HexBytes(bytes))
    }
}

impl Serialize for HexBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.encode_hex::<String>())
    }
}
