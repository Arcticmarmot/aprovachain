use rand_core::{OsRng, TryRngCore, OsError};

pub fn random_u128() -> Result<u128, OsError> {
    let mut bytes = [0u8; 16];
    OsRng.try_fill_bytes(&mut bytes)?;
    Ok(u128::from_le_bytes(bytes))
}
