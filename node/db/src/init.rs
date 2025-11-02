use rocksdb::{DB, Options};
use std::{fs};
use std::path::{PathBuf};
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use crate::controller::{kv_get, kv_put};
use crate::error::{DBError, Result};

/// TODO: 全局单例 DB
/// static DBH: Lazy<DB> = Lazy::new(|| open_db_inner().expect("open rocksdb"));
/// fn open_db_inner() -> Result<DB> {
///     let path = fixed_db_path();
///     fs::create_dir_all(&path).map_err(DBError::CreateDir)?;
///     let mut opts = Options::default();
///     // 仅做“能跑”的最小配置：首次无库时自动创建
///     opts.create_if_missing(true);
///     DB::open(&opts, path).map_err(DBError::Open)
/// }
///
/// TODO: TEMP DB 测试需要
/// fn open_temp_db() -> DB {
///     let dir = TempDir::new().expect("tmp dir");
///     let mut opts = Options::default();
///     opts.create_if_missing(true);
///     DB::open(&opts, dir.path()).expect("open temp db")
/// }
/// 交易：b"tx|" || tx_id(32) → BCS(TxPayload::Exec{...})
/// 代码：b"cd|" || code_hash(32) → source（或 meta + payload）
pub static DBH: Lazy<DB> = Lazy::new(|| open_db().expect("open rocksdb"));

pub fn db_init() {
    kv_put(b"test", b"1").unwrap();
    println!("{:?}", kv_get(b"test"));
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
    let db_dir = fixed_db_dir();
    fs::create_dir_all(&db_dir).map_err(DBError::DBDirCreate)?;
    // 创建数据库配置参数
    let mut opts = Options::default();
    opts.create_if_missing(true);
    DB::open(&opts, fixed_db_dir()).map_err(DBError::DBOpen)
}