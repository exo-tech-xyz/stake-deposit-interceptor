use std::str::FromStr;

use base64::Engine;
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

/// Deserialize Pubkey from a string
pub fn pubkey_from_str<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Pubkey::from_str(s).map_err(serde::de::Error::custom)
}

/// A human friendly Instruction that serializes Pubkeys to base58 and data to base64.
pub(crate) struct Instruction {
    program_id: Pubkey,
    accounts: Vec<AccountMeta>,
    data: Vec<u8>,
}

impl Instruction {
    pub fn from(instruction: solana_sdk::instruction::Instruction) -> Self {
        Instruction {
            program_id: instruction.program_id,
            accounts: instruction.accounts,
            data: instruction.data,
        }
    }
}

impl Serialize for Instruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Instruction", 3)?;

        state.serialize_field("programId", &self.program_id.to_string())?;

        let accounts_as_objects: Vec<_> = self.accounts.iter().map(|a| {
            serde_json::json!({
                "pubkey": a.pubkey.to_string(),
                "isSigner": a.is_signer,
                "isWritable": a.is_writable
            })
        }).collect();
        state.serialize_field("accounts", &accounts_as_objects)?;

        state.serialize_field(
            "data",
            &base64::engine::general_purpose::STANDARD.encode(&self.data),
        )?;

        state.end()
    }
}