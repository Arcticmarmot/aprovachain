use rocksdb::{DB, Options};
use std::{fs};
use std::path::{PathBuf};
use directories::ProjectDirs;
pub fn db_init() {
    let db_dir = create_db_dir();
    let db = DB::open_default(db_dir).unwrap();
    db.put(b"aprova", b"value").unwrap();
    match db.get(b"aprova") {
        Ok(Some(val)) => println!("the value is {:?}", val),
        Ok(None) => println!("not found"),
        Err(e) => eprintln!("{}", e)
    }
}

fn create_db_dir() -> PathBuf {
    /// directories 遵循 XDG 规范，忽略 qualifier 和 organization
    let mut proj_dir = ProjectDirs::from("com", "aprova", "aprova").expect("no proj dir");
    let db_dir = proj_dir.data_dir().join("rocksdb");
    fs::create_dir_all(&db_dir).expect("create db dir failed");
    db_dir
}