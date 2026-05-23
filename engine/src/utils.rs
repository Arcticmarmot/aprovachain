use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use risc0_zkvm::Receipt;

pub fn save_receipt_to_file<P: AsRef<Path>>(receipt: &Receipt, path: P) -> Result<()> {
    let bytes = bcs::to_bytes(receipt)
        .context("encode receipt to bcs failed")?;

    fs::write(path.as_ref(), bytes)
        .with_context(|| format!("write receipt file failed, path={}", path.as_ref().display()))?;

    Ok(())
}

pub fn load_receipt_from_file<P: AsRef<Path>>(path: P) -> Result<Receipt> {
    let bytes = fs::read(path.as_ref())
        .with_context(|| format!("read receipt file failed, path={}", path.as_ref().display()))?;

    let receipt: Receipt = bcs::from_bytes(&bytes)
        .with_context(|| format!("decode receipt from bcs failed, path={}", path.as_ref().display()))?;

    Ok(receipt)
}