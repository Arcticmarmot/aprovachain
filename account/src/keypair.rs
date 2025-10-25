use ed25519_dalek::{Signer, Verifier};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature};
use rand_core::OsRng;

pub type SignatureBytes = [u8; 64];

#[derive(Debug)]
pub struct Keypair {
    pub secret: SigningKey,
    pub public: VerifyingKey
}

impl Keypair {
    // random generate keypair
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let secret = SigningKey::generate(&mut csprng);
        let public = secret.verifying_key();
        Self {
            secret,
            public
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::keypair::Keypair;

    #[test]
    fn test_keypair_generate() {
        let kp = Keypair::new();
        println!("{:?}", kp);
    }
}