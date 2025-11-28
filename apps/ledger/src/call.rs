use account::address::{UserAddress};
use serde::{Deserialize, Serialize};
use spec::chain::ChainId;
use apps::ctr_io::{AccessSet, NamespaceKey};
use crate::error::Result;

pub const APP_NAME: &'static str = "ledger";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LedgerCall {
    Transfer { from: String, to: String, amount: u128 },
    QueryBalance { addr: String },
    Mint { to: String, amount: u128 },
    Burn { from: String, amount: u128 }
}

impl LedgerCall {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

fn address_to_entry_key(chain_id: ChainId, addr: &String) -> Result<NamespaceKey> {
    let addr = UserAddress::parse_bech32m_with_id(chain_id, addr)?;
    Ok(NamespaceKey {
        ns: "APP_NAME".to_string(),
        key: Vec::from(addr.to_bytes()),
    })
}

pub fn generate_access_set(chain_id: ChainId, input: Vec<u8>) -> Result<AccessSet> {
    let call = LedgerCall::try_decode_bcs(&input)?;
    let mut access_set = AccessSet::new();
    match call {
        LedgerCall::Transfer { from, to, .. } => {
            access_set.insert(address_to_entry_key(chain_id, &from)?);
            access_set.insert(address_to_entry_key(chain_id, &to)?);
        },
        LedgerCall::QueryBalance { addr } => {
            access_set.insert(address_to_entry_key(chain_id, &addr)?);
        },
        LedgerCall::Mint { to, amount } => {
            access_set.insert(address_to_entry_key(chain_id, &to)?);
        },
        LedgerCall::Burn { from, amount } => {
            access_set.insert(address_to_entry_key(chain_id, &from)?);
        }
    }
    Ok(access_set)
}


