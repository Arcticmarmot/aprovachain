use std::fs;
use std::path::Path;
use ed25519_dalek::{Signer, Verifier};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature};
use rand_core::OsRng;
use crate::address::{AccountAddress, AccountPrefix, UserAddress};
use crate::error::AccountError;


pub type Result<T> = std::result::Result<T, AccountError>;
pub type SigBytes = [u8; 64];
pub type SigningKeyBytes = [u8; 32];
pub type VerifyingKeyBytes = [u8; 32];
// 账户私钥
#[derive(Debug)]
pub struct AccountSigningKey(SigningKey);

// 账户公钥
#[derive(Debug)]
pub struct AccountVerifyingKey(VerifyingKey);

impl AccountSigningKey {
    pub fn from_bytes(signing_key_bytes: &SigningKeyBytes) -> Self {
        Self(SigningKey::from_bytes(signing_key_bytes))
    }

    pub fn to_bytes(&self) -> SigningKeyBytes {
        self.0.to_bytes()
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.0.sign(msg)
    }

    pub fn verifying_key(&self) -> AccountVerifyingKey {
        AccountVerifyingKey(self.0.verifying_key())
    }
}

impl AccountVerifyingKey {
    pub fn from_bytes(verifying_key_bytes: &VerifyingKeyBytes) -> Result<Self> {
        let verifying_key = VerifyingKey::from_bytes(verifying_key_bytes)
            .map_err(AccountError::InvalidVerifyingKey)?;
        Ok(Self(verifying_key))
    }

    pub fn to_bytes(&self) -> VerifyingKeyBytes {
        self.0.to_bytes()
    }

    pub fn verify(&self, msg: &[u8], sig: &Signature) -> Result<()> {
        self.0.verify(msg, sig).map_err(AccountError::SignatureVerify)?;
        Ok(())
    }

    pub fn is_valid_sig(&self, msg: &[u8], sig: &Signature) -> bool {
        self.verify(msg, sig).is_ok()
    }
}

#[derive(Debug)]
pub struct Keypair {
    pub signing_key: AccountSigningKey,
    pub verifying_key: AccountVerifyingKey
}

impl Keypair {
    // random generate keypair
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = AccountSigningKey(SigningKey::generate(&mut csprng));
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key
        }
    }

    pub fn save_address<T: AccountPrefix>(&self, dir: &Path, name: &str) -> Result<()> {
        let addr = AccountAddress::<T>::from(&self.verifying_key);
        fs::create_dir_all(dir).map_err(AccountError::CreateDir)?;
        let pathname = dir.join(format!("{name}"));
        fs::write(&pathname, addr.to_string()).map_err(AccountError::WriteHex)?;
        Ok(())
    }

    pub fn save_sk_hex(&self, dir: &Path, name: &str) -> Result<()>{
        let sk_hex = hex::encode(self.signing_key.to_bytes());
        fs::create_dir_all(dir).map_err(AccountError::CreateDir)?;
        let pathname = dir.join(format!("{name}.hex"));
        fs::write(&pathname, sk_hex).map_err(AccountError::WriteHex)?;
        Ok(())
    }

    pub fn save_vk_hex(&self, dir: &Path, name: &str) -> Result<()>{
        let vk_hex = hex::encode(self.verifying_key.to_bytes());
        fs::create_dir_all(dir).map_err(AccountError::CreateDir)?;
        let pathname = dir.join(format!("{name}.hex"));
        fs::write(&pathname, vk_hex).map_err(AccountError::WriteHex)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::keypair::Keypair;

    #[test]
    fn test_verify() {
        let keypair = Keypair::generate();
        let sk = keypair.signing_key;
        let pk = keypair.verifying_key;
        let msg = [100, 150, 200];
        let sig = sk.sign(&msg);
        assert_eq!(pk.verify(&msg, &sig).is_ok(), true);
    }

    #[test]
    fn test_is_valid_sig() {
        let keypair = Keypair::generate();
        let sk = keypair.signing_key;
        let pk = keypair.verifying_key;
        let msg = [100, 150, 200];
        let sig = sk.sign(&msg);
        let is_valid_sig = pk.is_valid_sig(&msg, &sig);
        assert_eq!(is_valid_sig, true);
    }
}