//! Address protocol implementation
use chain::registry;
use std::fmt;
use std::fmt::{Display, Formatter};
use sha2::{Digest};
use ed25519_dalek::Signer;
use bech32::{Hrp, Bech32m};
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

pub struct ChainAddress {
    chain_id: ChainId,
    addr: Address,
}

impl Address {
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; 20] {
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
        addr_bytes.copy_from_slice(&sha256(vk));
        Address(addr_bytes)
    }
}

impl ChainAddress {
    fn create(chain_id: ChainId, vk: &AccountVerifyingKey) -> Self {
        Self {
            chain_id,
            addr: Address::from(vk)
        }
    }

    fn hrp(&self) -> Hrp{
        registry::by_id(self.chain_id).hrp
    }

    fn to_bech32(&self) -> Result<String> {
        let addr_str = bech32::encode::<Bech32m>(self.hrp(), self.addr.as_bytes())
            .map_err(AccountError::Bech32Encode)?;
        Ok(addr_str)
    }
}

impl FromStr for ChainAddress {
    type Err = AccountError;
    fn from_str(s: &str) -> Result<Self> {
        let (hrp, data) = bech32::decode(s).map_err(AccountError::Bech32Decode)?;
        if hrp.as_str() != T::HRP {
            return Err(AccountError::HrpMismatch)
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&data);
        Ok(ChainAddress {

        })
    }
}

impl Display for ChainAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bech32_str = self.to_bech32().map_err(|_| fmt::Error)?;
        f.write_str(&bech32_str)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::address::{ChainAddress};
    use crate::keypair::Keypair;

    #[test]
    fn test_address() {
        /// TODO: 完善地址测试
        let keypair = Keypair::generate();
        // let pk = keypair.verifying_key;
        // let addr = AccountAddress::<UserAddress>::from(&pk);
        // let addr_bech32 = addr.to_bech32().unwrap();
        // let addr_str = addr.to_string();
        // assert_eq!(addr_bech32, addr_str);
        // println!("{}", addr_bech32);
        // let decoded_addr = AccountAddress::<UserAddress>::from_str(&addr_bech32).unwrap();
        // assert_eq!(decoded_addr, addr);
    }
}


