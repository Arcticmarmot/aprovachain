//! Address protocol implementation

use std::fmt;
use std::fmt::{Display, Formatter};
use sha2::{Digest, Sha256};
use ed25519_dalek::Signer;
use bech32::{Hrp, Bech32m};
use crate::error::AccountError;
use crate::keypair::AccountVerifyingKey;
use std::marker::PhantomData;
use std::str::FromStr;

pub type Result<T> = std::result::Result<T, AccountError>;

pub trait AccountPrefix {
    const HRP: &'static str;
}

#[derive(Debug, PartialEq)]
pub enum UserAddress {}
impl AccountPrefix for UserAddress {
    const HRP: &'static str = "user";
}

#[derive(Debug, PartialEq)]
pub struct AccountAddress<T: AccountPrefix>([u8; 20], PhantomData<T>);

impl<T: AccountPrefix> AccountAddress<T> {
    pub fn to_bech32(&self) -> Result<String> {
        let hrp = Hrp::parse(T::HRP)?;
        let addr = bech32::encode::<Bech32m>(hrp, &self.0).map_err(AccountError::Bech32Encode)?;
        Ok(addr)
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    pub fn from_bytes(b: [u8; 20]) -> Self {
        Self(b, PhantomData)
    }
}

impl<T: AccountPrefix> From<&AccountVerifyingKey> for AccountAddress<T> {
    fn from(vk: &AccountVerifyingKey) -> Self {
        let digest = Sha256::digest(vk.to_bytes());
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&digest[..20]);
        AccountAddress(bytes, PhantomData)
    }
}

impl<T: AccountPrefix> FromStr for AccountAddress<T> {
    type Err = AccountError;

    fn from_str(s: &str) -> Result<Self> {
        let (hrp, data) = bech32::decode(s).map_err(AccountError::Bech32Decode)?;
        if hrp.as_str() != T::HRP {
            return Err(AccountError::HrpMismatch)
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&data);
        Ok(AccountAddress(bytes, PhantomData))
    }
}

impl<T: AccountPrefix> Display for AccountAddress<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bech32_str = self.to_bech32().map_err(|_| fmt::Error)?;
        f.write_str(&bech32_str)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::address::{AccountAddress, UserAddress};
    use crate::keypair::Keypair;

    #[test]
    fn test_address() {
        let keypair = Keypair::generate();
        let pk = keypair.verifying_key;
        let addr = AccountAddress::<UserAddress>::from(&pk);
        let addr_bech32 = addr.to_bech32().unwrap();
        let addr_str = addr.to_string();
        assert_eq!(addr_bech32, addr_str);
        println!("{}", addr_bech32);
        let decoded_addr = AccountAddress::<UserAddress>::from_str(&addr_bech32).unwrap();
        assert_eq!(decoded_addr, addr);
    }
}


