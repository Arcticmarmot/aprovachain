//! Address protocol implementation
use sha2::{Digest, Sha256};
use ed25519_dalek::Signer;
use bech32::{hrp, Hrp, Bech32m};
use crate::error::AccountError;
use crate::keypair::AccountVerifyingKey;
use std::marker::PhantomData;

pub type Result<T> = std::result::Result<T, AccountError>;

pub trait AccountType {
    const HRP: &'static str;
}

pub enum UserAddress {}
impl AccountType for UserAddress {
    const HRP: &'static str = "user";
}
pub struct AccountAddress<T: AccountType>([u8; 20], PhantomData<T>);

impl<T: AccountType> AccountAddress<T> {
    fn to_bech32_address(&self) -> Result<String> {
        let hrp = Hrp::parse(T::HRP).unwrap();
        let addr = bech32::encode::<Bech32m>(hrp, &self.0).map_err(AccountError::Bech32EncodeError)?;
        Ok(addr)
    }
}

impl<T: AccountType> From<&AccountVerifyingKey> for AccountAddress<T> {
    fn from(vk: &AccountVerifyingKey) -> Self {
        let digest = Sha256::digest(vk.to_bytes());
        println!("{:?}", digest);
        let mut out = [0u8; 20];
        out.copy_from_slice(&digest[..20]);
        AccountAddress(out, PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use crate::address::{AccountAddress, UserAddress};
    use crate::keypair::Keypair;

    #[test]
    fn test_address() {
        let keypair = Keypair::generate();
        let sk = keypair.signing_key;
        let pk = keypair.verifying_key;
        let addr = AccountAddress::<UserAddress>::from(&pk);
        println!("{}", addr.to_bech32_address().unwrap());
    }
}


