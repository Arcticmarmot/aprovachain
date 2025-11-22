use sha2::{Digest, Sha256};
pub type Hash32 = [u8; 32];

pub const HASH32_ZERO: Hash32 = [0u8; 32];
pub fn sha256(data: impl AsRef<[u8]>) -> Hash32 {
    Sha256::digest(data.as_ref()).into()
}


/// 定义多段字节哈希宏
#[macro_export]
macro_rules! sha256_join {
    ($($x:expr),+$(,)?) => {{
        let mut hasher = Sha256::new();
        $( hasher.update($x.as_ref()); )+
        hasher.finalize().into()
    }};
}