use sha2::{Digest, Sha256};
pub type Hash32 = [u8; 32];

pub fn sha256(data: impl AsRef<[u8]>) -> Hash32 {
    Sha256::digest(data.as_ref()).into()
}

