use std::fs;
use std::path::Path;
use ed25519_dalek::{SecretKey, Signer, Verifier};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature};
use ed25519_dalek::ed25519::SignatureBytes;
use ed25519_dalek::PUBLIC_KEY_LENGTH;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use chain::spec::ChainId;
use crate::address::{ChainAddress};
use crate::error::AccountError;

pub type Result<T> = std::result::Result<T, AccountError>;
pub type AccountSignatureBytes = SignatureBytes;
pub type AccountSigningKeyBytes = SecretKey;
pub type AccountVerifyingKeyBytes = [u8; PUBLIC_KEY_LENGTH];

/// 签名
pub type AccountSignature = Signature;

/// 账户私钥
#[derive(Debug)]
pub struct AccountSigningKey(SigningKey);

/// 账户公钥
#[derive(Debug)]
pub struct AccountVerifyingKey(VerifyingKey);

impl AccountSigningKey {
    pub fn from_bytes(signing_key_bytes: &AccountSigningKeyBytes) -> Self {
        Self(SigningKey::from_bytes(signing_key_bytes))
    }

    pub fn to_bytes(&self) -> AccountSigningKeyBytes {
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
    pub fn from_bytes(verifying_key_bytes: &AccountVerifyingKeyBytes) -> Result<Self> {
        let verifying_key = VerifyingKey::from_bytes(verifying_key_bytes)
            .map_err(AccountError::InvalidVerifyingKey)?;
        Ok(Self(verifying_key))
    }

    pub fn to_bytes(&self) -> AccountVerifyingKeyBytes {
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
        let mut rng = OsRng;
        let signing_key = AccountSigningKey(SigningKey::generate(&mut rng));
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key
        }
    }

    pub fn save_address(&self, dir: &Path, filename: &str, chain_id: u64) -> Result<()> {
        let addr = ChainAddress::create(ChainId(chain_id), &self.verifying_key);
        fs::create_dir_all(dir).map_err(AccountError::CreateDir)?;
        let pathname = dir.join(format!("{filename}"));
        let addr_bech = addr.to_bech32()?;
        fs::write(&pathname, addr_bech).map_err(AccountError::WriteHex)?;
        Ok(())
    }

    pub fn save_sk_hex(&self, dir: &Path, filename: &str) -> Result<()>{
        let sk_hex = hex::encode(self.signing_key.to_bytes());
        fs::create_dir_all(dir).map_err(AccountError::CreateDir)?;
        let pathname = dir.join(format!("{filename}.hex"));
        fs::write(&pathname, sk_hex).map_err(AccountError::WriteHex)?;
        Ok(())
    }

    pub fn save_vk_hex(&self, dir: &Path, filename: &str) -> Result<()>{
        let vk_hex = hex::encode(self.verifying_key.to_bytes());
        fs::create_dir_all(dir).map_err(AccountError::CreateDir)?;
        let pathname = dir.join(format!("{filename}.hex"));
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