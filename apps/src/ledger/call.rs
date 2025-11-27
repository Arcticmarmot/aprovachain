use account::address::{UserAddress};
use serde::{Deserialize, Serialize};
use spec::ctr_io::{AccessSet, EntryKey};
use crate::ledger::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LedgerCall {
    Transfer { amount: u128, from: UserAddress, to: UserAddress },
    QueryBalance { addr: UserAddress }
}

impl LedgerCall {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

fn address_entry_key(addr: &UserAddress) -> EntryKey {
    EntryKey {
        cf: "ledger".to_string(),
        key: Vec::from(addr.to_bytes()),
    }
}

pub fn generate_access_set(input: Vec<u8>) -> Result<AccessSet> {
    let mut access_set = AccessSet::new();
    let call = LedgerCall::try_decode_bcs(&input)?;
    match call {
        LedgerCall::Transfer { from, to, .. } => {
            access_set.insert(address_entry_key(&from));
            access_set.insert(address_entry_key(&to));
        },
        LedgerCall::QueryBalance { addr } => {

        }
    }
    Ok(access_set)
}

