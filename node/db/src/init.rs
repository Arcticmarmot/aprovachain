use rocksdb::{DB, Options};
use std::{fs};
use std::fmt::Debug;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use tempfile::TempDir;
use crate::controller::{kv_get, kv_put};
use crate::error::{DBError, Result};

/// TODO: TEMP DB 测试需要
/// 交易：b"tx|" || tx_id(32) → BCS(TxPayload::Exec{...})
/// 代码：b"cd|" || code_hash(32) → source（或 meta + payload）
pub static TEMP_DIR: Lazy<TempDir> = Lazy::new(|| {
    tempfile::Builder::new()
        .prefix("rocksdb")
        .tempdir_in("/tmp")
        .expect("create temp dir")
});
pub static DBH: Lazy<DB> = Lazy::new(|| open_db().expect("open rocksdb"));

pub fn db_init() {
    kv_put(b"test", b"1").unwrap();
    println!("{:?}", kv_get(b"test"));
    kv_put(b"test", b"2").unwrap();
    println!("{:?}", kv_get(b"test"));
    let result = DBH.set_options(&[
        ("write_buffer_size", "262144"),                 // 256 KB
        ("level0_file_num_compaction_trigger", "2"),     // 更容易触发压实
    ]);
    println!("{:?}", result);
    match DBH.flush() {
        Ok(()) => eprintln!("flush OK（这说明你的目录其实还存在，或没按值传 TempDir）"),
        Err(e) => eprintln!("flush 失败（如预期，目录已被 unlink）：{e}"),
    }

}

fn temp_db_dir() -> impl AsRef<Path> + Debug {
    let db_dir = tempfile::Builder::new()
        .prefix("rocksdb")
        .tempdir_in("/tmp")
        .expect("create temp dir");
    db_dir
}

fn fixed_db_dir() -> PathBuf {
    // 数据库目录生成
    // linux: ~/.local/share/aprova/rocksdb (遵循 XDG 规范，忽略 qualifier 和 organization)
    // windows: C:\Users\<you>\AppData\Roaming\com\aprova\aprova
    // macOS: ~/Library/Application Support/com.aprova.aprova
    let proj_dir = ProjectDirs::from("com", "aprova", "aprova")
        .expect("no proj dir");
    let db_dir = proj_dir.data_dir().join("rocksdb");
    db_dir
}

pub fn open_db() -> Result<DB>{
    // 创建数据库文件目录
    let db_dir = temp_db_dir();
    println!("{:?}", db_dir);
    // 创建数据库配置参数
    let mut opts = Options::default();
    opts.create_if_missing(true);
    DB::open(&opts, db_dir).map_err(DBError::DBOpen)
}

pub fn close_db() -> Result<()> {
    let _ = DB::destroy(&Options::default(), temp_db_dir());
    Ok(())
}