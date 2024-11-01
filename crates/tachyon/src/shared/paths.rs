use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use matrix_sdk::ruma::UserId;
use msnp::shared::models::uuid::Uuid;
use crate::shared::paths;

pub fn get_tachyon_path() -> Option<ProjectDirs> {
    directories::ProjectDirs::from("", "","Tachyon")
}

pub fn create_dirs(dirs: &ProjectDirs) {
    let config_path = dirs.config_dir();
    let data_path = dirs.data_dir();
    let local_data_path = dirs.data_local_dir();

    create_dir(config_path);
    create_dir(data_path);
    create_dir(local_data_path);
}

fn create_dir(path: &Path) {
    println!("Path: {:?}", &path);
    if let Err(e) = path.read_dir() {
        println!("Could'nt read Tachyon Folder, creating it...{:?}", e);
        fs::create_dir_all(&path).expect("to work");
    }
}


pub fn sanitize_user_id(user_id: &UserId) -> String {
    if cfg!(debug_assertions) {
        let user_id = user_id.to_string();
        let no_prefix = user_id.trim_start_matches("@");
        no_prefix.replace(":", "_")
    } else {
        Uuid::from_seed(user_id.as_str()).to_string()
    }
}

pub fn get_user_data(user_id: &UserId) -> Option<PathBuf> {
    Some(get_tachyon_path()?.data_local_dir().join(sanitize_user_id(user_id)))
}

pub fn get_store_path(user_id: &UserId) -> Option<PathBuf> {
    Some(get_user_data(user_id)?.join("store"))
}