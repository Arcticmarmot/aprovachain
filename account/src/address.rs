use std::fmt::{Debug, Formatter};
use sha2::Digest;
use spec::registry;
use bech32::{Bech32m};
use crate::error::AccountError;
use crate::keypair::AccountVerifyingKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use spec::chain::ChainId;
use primitives::hash::{sha256, Hash32};
use primitives::sha256_join;

pub type Result<T> = std::result::Result<T, AccountError>;

pub type AddressBytes = [u8; 20];
pub type ChainAddrBytes = [u8; 28];

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address(AddressBytes);

impl Address {
    pub fn to_bytes(&self) -> AddressBytes {
        self.0
    }

    pub fn as_bytes(&self) -> &AddressBytes {
        &self.0
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<AddressBytes> for Address {
    fn from(b: AddressBytes) -> Self {
        Self(b)
    }
}

impl From<Address> for AddressBytes {
    fn from(addr: Address) -> Self {
        addr.0
    }
}

impl From<&AccountVerifyingKey> for Address {
    fn from(vk: &AccountVerifyingKey) -> Self {
        let mut buf: AddressBytes = [0u8; 20];
        buf.copy_from_slice(&sha256(vk.to_bytes())[..20]);
        Address(buf)
    }
}

#[derive(Debug, Clone)]
pub struct UserAddress {
    pub chain_id: ChainId,
    pub addr: Address,
}

impl UserAddress {
    pub fn new(chain_id: ChainId, addr: Address) -> Self {
        Self {
            chain_id,
            addr
        }
    }

    pub fn from_vk(chain_id: ChainId, vk: &AccountVerifyingKey) -> Self {
        Self {
            chain_id,
            addr: Address::from(vk)
        }
    }

    pub fn addr_bytes(&self) -> AddressBytes  {
        self.addr.to_bytes()
    }

    pub fn to_bytes(&self) -> ChainAddrBytes {
        compose_chain_addr_bytes(self.chain_id, self.addr)
    }

    pub fn from_bytes(bytes: ChainAddrBytes) -> Self {
        let (chain_id, addr) = depose_chain_addr_bytes(bytes);
        Self::new(chain_id, addr)
    }

    pub fn to_bech32m(&self) -> Result<String> {
        let hrp = registry::hrp_by_id(self.chain_id).ok_or(AccountError::HrpNotInRegistry)?;
        let addr_bech32m = bech32::encode::<Bech32m>(hrp, self.addr.as_ref())
            .map_err(AccountError::Bech32Encode)?;
        Ok(addr_bech32m)
    }

    pub fn parse_bech32m_with_id(chain_id: ChainId, s: &str) -> Result<Self> {
        let (hrp, data) = bech32::decode(s).map_err(AccountError::Bech32Decode)?;
        let registry_hrp = registry::hrp_by_id(chain_id).ok_or(AccountError::HrpNotInRegistry)?;
        if hrp.as_str() != registry_hrp.as_str() {
            return Err(AccountError::HrpMismatch);
        }
        let mut buf: AddressBytes = [0u8; 20];
        buf.copy_from_slice(&data);
        Ok(UserAddress {
            chain_id,
            addr: Address::from(buf)
        })
    }
}

#[derive(Clone)]
pub struct ContractAddress {
    pub chain_id: ChainId,
    pub addr: Address,
}

impl Debug for ContractAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} ", self.to_bech32m().unwrap())
    }
}

impl ContractAddress {
    pub fn new(chain_id: ChainId, addr: Address) -> Self {
        Self {
            chain_id,
            addr
        }
    }

    pub fn create(chain_id: ChainId, vk: &AccountVerifyingKey, nonce: u128) -> Self {
        let mut addr_bytes: AddressBytes = [0u8; 20];
        let chain_id_bytes = chain_id.0.to_be_bytes();
        let vk_bytes = vk.to_bytes();
        let nonce_bytes = nonce.to_be_bytes();
        let addr_hash: Hash32 = sha256_join!(chain_id_bytes, vk_bytes, nonce_bytes);
        addr_bytes.copy_from_slice(&addr_hash[..20]);
        Self {
            chain_id,
            addr: Address::from(addr_bytes)
        }
    }

    pub fn addr_bytes(&self) -> AddressBytes  {
        self.addr.to_bytes()
    }

    pub fn to_bytes(&self) -> ChainAddrBytes {
        compose_chain_addr_bytes(self.chain_id, self.addr)
    }

    pub fn from_bytes(bytes: ChainAddrBytes) -> Self {
        let (chain_id, addr) = depose_chain_addr_bytes(bytes);
        Self::new(chain_id, addr)
    }

    pub fn to_bech32m(&self) -> Result<String> {
        let hrp = registry::ctr_hrp_by_id(self.chain_id).ok_or(AccountError::HrpNotInRegistry)?;
        let addr_str = bech32::encode::<Bech32m>(hrp, self.addr.as_ref())
            .map_err(AccountError::Bech32Encode)?;
        Ok(addr_str)
    }

    pub fn parse_bech32m_with_id(chain_id: ChainId, s: &str) -> Result<Self> {
        let (hrp, data) = bech32::decode(s).map_err(AccountError::Bech32Decode)?;
        let registry_hrp = registry::ctr_hrp_by_id(chain_id).ok_or(AccountError::HrpNotInRegistry)?;
        if hrp.as_str() != registry_hrp.as_str() {
            return Err(AccountError::HrpMismatch);
        }
        let mut buf: AddressBytes = [0u8; 20];
        buf.copy_from_slice(&data);
        Ok(ContractAddress {
            chain_id,
            addr: Address::from(buf)
        })
    }
}

fn compose_chain_addr_bytes(chain_id: ChainId, addr: Address) -> ChainAddrBytes {
    let mut buf: ChainAddrBytes = [0u8; 28];
    buf[..8].copy_from_slice(&chain_id.0.to_be_bytes());
    buf[8..].copy_from_slice(addr.as_bytes());
    buf
}

fn depose_chain_addr_bytes(bytes: ChainAddrBytes) -> (ChainId, Address) {
    let mut chain_id_bytes = [0u8; 8];
    let mut addr_bytes: AddressBytes = [0u8; 20];
    chain_id_bytes.copy_from_slice(&bytes[..8]);
    addr_bytes.copy_from_slice(&bytes[8..]);
    let chain_id = ChainId(u64::from_be_bytes(chain_id_bytes));
    let addr = Address::from(addr_bytes);
    (chain_id, addr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair::Keypair;

    #[test]
    fn test_address() {
        let keypair = Keypair::generate();
        let pk = keypair.verifying_key;
        let addr = Address::from(&pk);
        let chain_id = ChainId(1000);
        let chain_addr = UserAddress {
            chain_id,
            addr,
        };
        let addr_bech32 = chain_addr.to_bech32m().unwrap();
        println!("{}", addr_bech32);
    }
}


