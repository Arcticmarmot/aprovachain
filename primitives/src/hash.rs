use sha2::{Digest, Sha256};
pub type Hash32 = [u8; 32];

pub fn sha256(data: impl AsRef<[u8]>) -> Hash32 {
    Sha256::digest(data.as_ref()).into()
}


/// TODO: 完成sha256_concat
pub fn sha256_concat(chunks: Vec<impl AsRef<[u8]>>) -> Hash32 {
    let mut hasher = Sha256::new();
    for c in chunks {
        hasher.update(c.as_ref());
    }
    hasher.finalize().into()
}