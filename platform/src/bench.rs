use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
};
use anyhow::{Context,Result};
use crate::file::aprova_proj_dir;

/// 你测试里给一个文件名，比如 "fib_prove.csv"
/// 这里统一落到 aprova 的 data_local_dir/bench/ 下
pub fn bench_csv_path(filename: &str) -> PathBuf {
    let proj = aprova_proj_dir().expect("proj dir not found");
    proj.data_local_dir().join("bench").join(filename)
}

/// 开始一次 bench：创建目录 + truncate 文件 + 写 header
pub fn bench_csv_begin(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create bench dir: {}", parent.display()))?;
    }
    // truncate 清空
    fs::write(path, "prove_ms\n")
        .with_context(|| format!("init bench csv: {}", path.display()))?;
    Ok(())
}

/// 追加一行（线性写；简单起见直接手写 CSV 行）
/// 如果你担心字段里有逗号，就用 csv::Writer（见下方替代版本）
pub fn bench_csv_append(path: &Path, prove_ms: u64) -> Result<()> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open bench csv append: {}", path.display()))?;

    use std::io::Write;
    writeln!(f, "{},", prove_ms)
        .context("write bench csv row")?;
    Ok(())
}
