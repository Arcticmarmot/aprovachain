use rocksdb::{DB, Options};
use std::{fs};
use std::path::{PathBuf};
use directories::ProjectDirs;
use crate::controller::{kv_get, kv_put};
use crate::error::DBError;

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

type Result<T> = std::result::Result<T, DBError>;

pub fn db_init() {
    create_db_dir();
    let _ = kv_put(b"image_id", b"123").unwrap();
    let value = kv_get(b"image_id");
    println!("{:?}", value);
}

fn create_db_dir() {
    let db_dir = db_dir_fixed();
    fs::create_dir_all(&db_dir).expect("create db dir failed");
}

fn db_dir_fixed() -> PathBuf {
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
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, db_dir_fixed()).map_err(DBError::DBOpen)?;
    Ok(db)
}