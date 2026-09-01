use std::fs;
use std::path::Path;
use directories::ProjectDirs;

pub fn get_tachyon_path() -> ProjectDirs {
    directories::ProjectDirs::from("", "","Tachyon").expect("Could not resolve root configuration directory")
}

pub fn create_dirs(dirs: &ProjectDirs) {
    let config_path = dirs.config_dir();
    let data_path = dirs.data_dir();
    let local_data_path = dirs.data_local_dir();

    create_dir(config_path);
    create_dir(data_path);
    create_dir(local_data_path);
}

pub fn create_dir(path: &Path) {
    println!("Path: {:?}", &path);
    if let Err(e) = path.read_dir() {
        println!("Could'nt read Tachyon Folder, creating it...{:?}", e);
        fs::create_dir_all(&path).expect("to work");
    }
}

