pub fn u64_to_be_vec(value: u64) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

pub fn u64_from_be_slice(vec: &[u8]) -> u64 {
    let mut out = [0u8; 8];
    out.copy_from_slice(vec);
    u64::from_be_bytes(out)
}