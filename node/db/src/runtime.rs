//! 数据库初始化和析构方法
//! ArcSwapOption 全局句柄交换（原子操作，线程安全无锁，不保证读数据是最新的）

use rocksdb::{DB, Options};
use std::path::{PathBuf};
use std::sync::{Arc};
use directories::ProjectDirs;
use tempfile::TempDir;
use crate::error::{DBError, Result};
use arc_swap::{ArcSwapOption};
use once_cell::sync::{Lazy, OnceCell};

#[derive(Debug, Copy, Clone)]
pub enum DBMode {
    Ephemeral,
    Persistent
}
/// DB_PATH 仅记录第一次写入的路径，OnceCell保证不可删除
pub static DB_PATH: OnceCell<PathBuf> = OnceCell::new();

/// TEMP_DIR 临时文件夹对象
pub static TEMP_DIR: Lazy<ArcSwapOption<TempDir>> = Lazy::new(|| ArcSwapOption::from(None));

/// DBH 句柄对象
pub static DBH: Lazy<ArcSwapOption<DB>> = Lazy::new(|| ArcSwapOption::from(None));

pub fn init_db(mode: DBMode) -> Result<()> {
    let (path, temp_dir_wrap): (PathBuf, Option<Arc<TempDir>>) = match mode {
        DBMode::Ephemeral => {
            let db_dir = temp_db_dir()?;
            (PathBuf::from(db_dir.path()), Some(Arc::new(db_dir)))
        },
        DBMode::Persistent => {
            let db_dir = fixed_db_dir()?;
            (db_dir, None)
        }
    };

    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, &path).map_err(DBError::DBOpen)?;
    DBH.store(Some(Arc::new(db)));

    DB_PATH.set(path).expect("DB_PATH set failed");
    TEMP_DIR.store(temp_dir_wrap);
    Ok(())
}

#[inline]
pub fn dbh() -> Result<Arc<DB>> {
    DBH.load_full().ok_or(DBError::DBNotInit)
}

pub fn close_db(mode: DBMode) -> Result<()> {
    if let Some(db) = DBH.swap(None) {
        db.flush().expect("DB flush failed");
        db.flush_wal(true).expect("DB flush wal failed");
        db.cancel_all_background_work(true);
        drop(db);
    }

    match mode {
        DBMode::Ephemeral => {
            let path = DB_PATH.get().unwrap();
            DB::destroy(&Options::default(), path).expect("DB destroy failed");
        },
        _ => {}
    }

    if let Some(temp_dir) = TEMP_DIR.swap(None) {
        drop(temp_dir);
    }
    Ok(())
}

fn temp_db_dir() -> Result<TempDir> {
    // 创建数据库文件目录
    tempfile::Builder::new()
        .prefix("rocksdb")
        .tempdir_in("/tmp")
        .map_err(DBError::DBTempDirCreate)
}

fn fixed_db_dir() -> Result<PathBuf> {
    // 数据库目录生成
    // linux: ~/.local/share/aprova/rocksdb (遵循 XDG 规范，忽略 qualifier 和 organization)
    // windows: C:\Users\<you>\AppData\Roaming\com\aprova\aprova
    // macOS: ~/Library/Application Support/com.aprova.aprova
    let proj_dir = ProjectDirs::from("com", "aprova", "aprova")
        .ok_or(DBError::DBFixedDirCreate)?;
    let db_dir = proj_dir.data_dir().join("rocksdb");
    Ok(db_dir)
}
