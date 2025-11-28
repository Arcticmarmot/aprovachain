use std::path::PathBuf;
use directories::ProjectDirs;

pub fn aprova_proj_dir() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "aprova", "aprova")
}

pub fn load_node_sk_path() -> PathBuf {
    let proj = aprova_proj_dir().expect("proj dir not found");
    proj.data_local_dir().join("keypair").join("../../node").join("signing-key.hex")
}

pub fn load_user_sk_path() -> PathBuf {
    let proj = aprova_proj_dir().expect("proj dir not found");
    proj.data_local_dir().join("keypair").join("../../node").join("signing-key.hex")
}


