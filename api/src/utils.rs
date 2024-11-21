use std::str::FromStr;

use serde::{Deserialize, Deserializer};
use solana_sdk::pubkey::Pubkey;

/// Deserialize Pubkey from a string
pub fn pubkey_from_str<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Pubkey::from_str(s).map_err(serde::de::Error::custom)
}