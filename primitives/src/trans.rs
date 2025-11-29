pub fn u128_to_be_vec(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

pub fn u128_from_be_slice(vec: &[u8]) -> u128 {
    let mut out = [0u8; 16];
    out.copy_from_slice(vec);
    u128::from_be_bytes(out)
}