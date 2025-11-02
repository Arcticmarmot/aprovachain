use rocksdb::{DB, Options};
use std::path::{Path, PathBuf};
use std::sync::{Arc};
use directories::ProjectDirs;
use tempfile::TempDir;
use crate::controller::{kv_get, kv_put};
use crate::error::{DBError, Result};
use arc_swap::{ArcSwap, ArcSwapOption};
use once_cell::sync::OnceCell;

/// TODO: TEMP DB 测试需要
/// 交易：b"tx|" || tx_id(32) → BCS(TxPayload::Exec{...})
/// 代码：b"cd|" || code_hash(32) → source（或 meta + payload）
pub enum DBMode {
    Ephemeral,
    Persistent
}
pub static TEMP_DIR: OnceCell<ArcSwapOption<TempDir>> = OnceCell::new();
pub static DB_PATH: OnceCell<PathBuf> = OnceCell::new();
pub static DBH: OnceCell<ArcSwapOption<DB>> = OnceCell::new();

pub fn db_init(mode: DBMode) -> Result<()> {
    let (path, temp_dir_opt): (PathBuf, Option<TempDir>) = match mode {
        DBMode::Ephemeral => {
            let db_dir = temp_db_dir();
            (PathBuf::from(db_dir.path()), Some(db_dir))
        },
        DBMode::Persistent => {
            let db_dir = fixed_db_dir();
            (db_dir, None)
        }
    };
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, &path).expect("db set failed");
    DBH.set(ArcSwapOption::new(Some(Arc::new(db)))).expect("DBH set failed");
    DB_PATH.set(path).expect("db path set failed");
    if let Some(opt) = temp_dir_opt {
        TEMP_DIR.set(ArcSwapOption::new(Some(Arc::new(opt)))).expect("temp dir set failed");
    }
    kv_put(b"hello", b"world");
    println!("{:?}", kv_get(b"hello"));
    Ok(())
}



fn temp_db_dir() -> TempDir {
    // 创建数据库文件目录
    tempfile::Builder::new()
        .prefix("rocksdb")
        .tempdir_in("/tmp")
        .expect("create temp dir")
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

pub fn close_db() -> Result<()> {
    let db = DBH.get().unwrap().load_full().unwrap();
    db.flush().expect("TODO: panic message");
    db.flush_wal(true).expect("TODO: panic message");
    db.cancel_all_background_work(true);
    drop(db);
    DBH.get().unwrap().swap(None);
    // DB::destroy(&Options::default(), DB_PATH.get().unwrap()).unwrap();
    if let Some(temp_dir) = TEMP_DIR.get().unwrap().swap(None) {
        drop(temp_dir)
    }
    Ok(())
}

// pub fn db_init() {
//     kv_put(b"test", b"1").unwrap();
//     println!("{:?}", kv_get(b"test"));
//     kv_put(b"test", b"2").unwrap();
//     println!("{:?}", kv_get(b"test"));
//     let db = DBH.load_full().take().unwrap();
//     let result = db.set_options(&[
//         ("write_buffer_size", "262144"),                 // 256 KB
//         ("level0_file_num_compaction_trigger", "2"),     // 更容易触发压实
//     ]);
//     println!("{:?}", result);
//     match db.flush() {
//         Ok(()) => eprintln!("flush OK（这说明你的目录其实还存在，或没按值传 TempDir）"),
//         Err(e) => eprintln!("flush 失败（如预期，目录已被 unlink）：{e}"),
//     }
// }