//! Address protocol implementation
use chain::registry;
use std::fmt;
use std::fmt::{Display, Formatter};
use sha2::{Digest};
use ed25519_dalek::Signer;
use bech32::{Bech32};
use crate::error::AccountError;
use crate::keypair::AccountVerifyingKey;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use chain::spec::ChainId;
use primitives::hash::sha256;

pub type Result<T> = std::result::Result<T, AccountError>;

pub type AddressBytes = [u8; 20];

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Address(AddressBytes);

#[derive(Debug)]
pub struct ChainAddress {
    pub chain_id: ChainId,
    pub addr: Address,
}

impl Address {
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; 20] {
        self.0
    }

    pub fn to_bytes(&self) -> [u8; 20] {
        self.0
    }
}

impl From<[u8; 20]> for Address {
    fn from(b: [u8; 20]) -> Self {
        Self(b)
    }
}

impl From<&AccountVerifyingKey> for Address {
    fn from(vk: &AccountVerifyingKey) -> Self {
        let mut addr_bytes: AddressBytes = [0u8; 20];
        addr_bytes.copy_from_slice(&sha256(vk.to_bytes())[..20]);
        Address(addr_bytes)
    }
}

impl ChainAddress {
    pub fn create(chain_id: ChainId, vk: &AccountVerifyingKey) -> Self {
        Self {
            chain_id,
            addr: Address::from(vk)
        }
    }

    pub fn to_bech32(&self) -> Result<String> {
        let hrp = registry::hrp_by_id(self.chain_id);
        if hrp == None {
            return Err(AccountError::HrpMismatch);
        }
        let addr_str = bech32::encode::<Bech32>(hrp.unwrap(), self.addr.as_bytes())
            .map_err(AccountError::Bech32Encode)?;
        Ok(addr_str)
    }

    pub fn try_from_str_with_id(chain_id: ChainId, s: &str) -> Result<Self> {
        let (hrp, data) = bech32::decode(s).map_err(AccountError::Bech32Decode)?;
        let registry_hrp = registry::hrp_by_id(chain_id);
        if registry_hrp == None || hrp.as_str() != registry_hrp.unwrap().as_str() {
            return Err(AccountError::HrpMismatch);
        }
        let mut bytes: AddressBytes = [0u8; 20];
        bytes.copy_from_slice(&data);
        Ok(ChainAddress {
            chain_id,
            addr: Address::from(bytes)
        })
    }

    pub fn to_bytes(&self) -> [u8; 20]  {
        self.addr.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair::Keypair;

    #[test]
    fn test_address() {
        /// TODO: 完善地址测试
        let keypair = Keypair::generate();
        let pk = keypair.verifying_key;
        let addr = Address::from(&pk);
        let chain_id = ChainId(1000);
        let chain_addr = ChainAddress {
            chain_id,
            addr,
        };
        let addr_bech32 = chain_addr.to_bech32().unwrap();
        println!("{}", addr_bech32);
    }
}


