use std::{fs, path};

pub fn init(path: String) {
    println!("Creating {}", path);
    fs::create_dir(path.clone()).unwrap();
    fs::create_dir(path::Path::new(&path).join("src")).unwrap();
}
