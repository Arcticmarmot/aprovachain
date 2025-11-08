use sha2::Digest;
use chain::registry;
use bech32::{Bech32m};
use crate::error::AccountError;
use crate::keypair::AccountVerifyingKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use chain::spec::ChainId;
use primitives::hash::{sha256, Hash32};
use primitives::sha256_join;

pub type Result<T> = std::result::Result<T, AccountError>;

pub type AddressBytes = [u8; 20];

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Address(AddressBytes);

impl Address {
    pub fn into_bytes(self) -> AddressBytes {
        self.0
    }

    pub fn to_bytes(&self) -> AddressBytes {
        self.0
    }
}

impl From<AddressBytes> for Address {
    fn from(b: AddressBytes) -> Self {
        Self(b)
    }
}

impl AsRef<AddressBytes> for Address {
    fn as_ref(&self) -> &AddressBytes {
        &self.0
    }
}

impl From<Address> for AddressBytes {
    fn from(addr: Address) -> Self {
        addr.0
    }
}

impl From<&AccountVerifyingKey> for Address {
    fn from(vk: &AccountVerifyingKey) -> Self {
        let mut addr_bytes: AddressBytes = [0u8; 20];
        addr_bytes.copy_from_slice(&sha256(vk.to_bytes())[..20]);
        Address(addr_bytes)
    }
}

#[derive(Debug)]
pub struct UserAddress {
    pub chain_id: ChainId,
    pub addr: Address,
}

impl UserAddress {
    pub fn create(chain_id: ChainId, addr: Address) -> Self {
        Self {
            chain_id,
            addr
        }
    }

    pub fn create_from_vk(chain_id: ChainId, vk: &AccountVerifyingKey) -> Self {
        Self {
            chain_id,
            addr: Address::from(vk)
        }
    }

    pub fn create_from_bytes(chain_id: ChainId, addr_bytes: AddressBytes) -> Self {
        Self {
            chain_id,
            addr: Address::from(addr_bytes)
        }
    }

    pub fn to_bech32m(&self) -> Result<String> {
        let hrp = registry::hrp_by_id(self.chain_id).ok_or(AccountError::HrpNotInRegistry)?;
        let addr_str = bech32::encode::<Bech32m>(hrp, self.addr.as_ref())
            .map_err(AccountError::Bech32Encode)?;
        Ok(addr_str)
    }

    pub fn try_from_str_with_id(chain_id: ChainId, s: &str) -> Result<Self> {
        let (hrp, data) = bech32::decode(s).map_err(AccountError::Bech32Decode)?;
        let registry_hrp = registry::hrp_by_id(chain_id).ok_or(AccountError::HrpNotInRegistry)?;
        if hrp.as_str() != registry_hrp.as_str() {
            return Err(AccountError::HrpMismatch);
        }
        let mut bytes: AddressBytes = [0u8; 20];
        bytes.copy_from_slice(&data);
        Ok(UserAddress {
            chain_id,
            addr: Address::from(bytes)
        })
    }

    pub fn to_bytes(&self) -> [u8; 20]  {
        self.addr.to_bytes()
    }
}

#[derive(Debug)]
pub struct ContractAddress {
    pub chain_id: ChainId,
    pub addr: Address,
}

impl ContractAddress {
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

    pub fn create_from_bytes(chain_id: ChainId, addr_bytes: AddressBytes) -> Self {
        Self {
            chain_id,
            addr: Address::from(addr_bytes)
        }
    }

    pub fn to_bech32m(&self) -> Result<String> {
        let hrp = registry::ctr_hrp_by_id(self.chain_id).ok_or(AccountError::HrpNotInRegistry)?;
        let addr_str = bech32::encode::<Bech32m>(hrp, self.addr.as_ref())
            .map_err(AccountError::Bech32Encode)?;
        Ok(addr_str)
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


