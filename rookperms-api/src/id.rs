use std::fmt;
use std::str::FromStr;

use serde::de::{Error as DeError, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerId(u128);

impl PlayerId {
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    pub const fn from_parts(high: u64, low: u64) -> Self {
        Self(((high as u128) << 64) | low as u128)
    }

    pub const fn as_u128(self) -> u128 {
        self.0
    }

    pub const fn parts(self) -> (u64, u64) {
        ((self.0 >> 64) as u64, self.0 as u64)
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = format!("{:032x}", self.0);
        write!(
            f,
            "{}-{}-{}-{}-{}",
            &hex[0..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..32]
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerIdParseError;

impl fmt::Display for PlayerIdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid player uuid")
    }
}

impl std::error::Error for PlayerIdParseError {}

impl FromStr for PlayerId {
    type Err = PlayerIdParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hex: String = value.chars().filter(|c| *c != '-').collect();
        if hex.len() != 32 {
            return Err(PlayerIdParseError);
        }
        u128::from_str_radix(&hex, 16)
            .map(Self)
            .map_err(|_| PlayerIdParseError)
    }
}

impl Serialize for PlayerId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PlayerId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::from_str(&raw)
            .map_err(|_| D::Error::invalid_value(Unexpected::Str(&raw), &"a player uuid"))
    }
}
