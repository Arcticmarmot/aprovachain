use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
};
use anyhow::{Context,Result};
use crate::file::aprova_proj_dir;
use std::io::Write;

pub fn bench_receipt_path(filename: &str, prove_scheme: &str) -> PathBuf {
    let proj = aprova_proj_dir().expect("proj dir not found");
    let filename = proj.data_local_dir().join("receipt").join(prove_scheme).join(format!("{}.bsc", filename));
    if let Some(parent) = filename.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create bench dir: {}", parent.display())).expect("create receipt dir failed");
    }
    filename
}

pub fn bench_csv_path(filename: &str) -> PathBuf {
    let proj = aprova_proj_dir().expect("proj dir not found");
    proj.data_local_dir().join("bench").join(format!("{}.csv", filename))
}

pub fn bench_smallbank_csv_path(filename: &str) -> PathBuf {
    let proj = aprova_proj_dir().expect("proj dir not found");
    proj.data_local_dir().join("smallbank").join(format!("{}.csv", filename))
}

pub fn bench_prove_receipt_csv_begin(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create bench dir: {}", parent.display()))?;
    }
    // cycles,prove_time,receipt_size
    fs::write(path, "cycles,prove_time,receipt_size\n")
        .with_context(|| format!("init bench csv: {}", path.display()))?;
    Ok(())
}

pub fn bench_prove_receipt_csv_append(path: &Path, cycles: u64, prove_time: u128, receipt_size: usize) -> Result<()> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open bench csv append: {}", path.display()))?;
    writeln!(f, "{},{},{},", cycles, prove_time, receipt_size)
        .context("write bench csv row")?;
    Ok(())
}

pub fn bench_verify_receipt_csv_begin(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create bench dir: {}", parent.display()))?;
    }
    // cycles,prove_time,receipt_size
    fs::write(path, "cycles,verify_time\n")
        .with_context(|| format!("init bench csv: {}", path.display()))?;
    Ok(())
}

pub fn bench_verify_receipt_csv_append(path: &Path, cycles: u64, verify_time: u128) -> Result<()> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open bench csv append: {}", path.display()))?;
    writeln!(f, "{},{},", cycles, verify_time)
        .context("write bench csv row")?;
    Ok(())
}

pub fn bench_validate_block_csv_begin(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create bench dir: {}", parent.display()))?;
    }
    // cycles,prove_time,receipt_size
    fs::write(path, "size,validate_time\n")
        .with_context(|| format!("init bench csv: {}", path.display()))?;
    Ok(())
}

pub fn bench_validate_block_csv_append(path: &Path, size: usize, validate_time: u128) -> Result<()> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open bench csv append: {}", path.display()))?;
    writeln!(f, "{},{},", size, validate_time)
        .context("write bench csv row")?;
    Ok(())
}

pub fn bench_smallbank_csv_begin(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create bench dir: {}", parent.display()))?;
    }
    // cycles,prove_time,receipt_size
    fs::write(path, "prover_id,code,count\n")
        .with_context(|| format!("init bench csv: {}", path.display()))?;
    Ok(())
}

pub fn bench_smallbank_csv_append(path: &Path, prover_id: String, code: String, count: usize) -> Result<()> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open bench csv append: {}", path.display()))?;
    writeln!(f, "{},{},{},", prover_id, code, count)
        .context("write bench csv row")?;
    Ok(())
}

