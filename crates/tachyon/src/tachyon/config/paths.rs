use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use matrix_sdk::ruma::UserId;
use msnp::shared::models::uuid::Uuid;

pub struct TachyonDirs {
    config_dir: PathBuf,
    data_dir: PathBuf,
    data_local_dir: PathBuf,
}

impl TachyonDirs {
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn data_local_dir(&self) -> &Path {
        &self.data_local_dir
    }
}

pub fn get_tachyon_path() -> TachyonDirs {
    if let Ok(home) = std::env::var("TACHYON_HOME") {
        if !home.trim().is_empty() {
            let home = PathBuf::from(home);
            let local = home.join("local");
            return TachyonDirs { config_dir: home.clone(), data_dir: home, data_local_dir: local };
        }
    }

    let dirs = ProjectDirs::from("", "","Tachyon").expect("Could not resolve root configuration directory");
    TachyonDirs {
        config_dir: dirs.config_dir().to_path_buf(),
        data_dir: dirs.data_dir().to_path_buf(),
        data_local_dir: dirs.data_local_dir().to_path_buf(),
    }
}

pub fn create_dirs(dirs: &TachyonDirs) {
    create_dir(dirs.config_dir());
    create_dir(dirs.data_dir());
    create_dir(dirs.data_local_dir());
}

pub fn create_dir(path: &Path) {
    println!("Path: {:?}", path);
    if let Err(e) = path.read_dir() {
        println!("Could'nt read Tachyon Folder, creating it...{:?}", e);
        fs::create_dir_all(path).expect("to work");
    }
}

pub fn credentials_db_path() -> PathBuf {
    get_tachyon_path().data_local_dir().join("credentials.db")
}

pub fn sanitize_user_id(user_id: &UserId) -> String {
    Uuid::from_seed(user_id.as_str()).to_string()
}

pub fn get_user_data(user_id: &UserId) -> PathBuf {
    get_tachyon_path().data_local_dir().join(sanitize_user_id(user_id))
}

pub fn get_store_path(user_id: &UserId) -> PathBuf {
    get_user_data(user_id).join("store")
}
