use std::{fs, io::Write, path};

pub fn init(path: String) {
    println!("Creating {path}");
    fs::create_dir(path.clone()).unwrap();
    fs::create_dir(path::Path::new(&path).join("src")).unwrap();
    let mut conf_file = fs::File::create(path::Path::new(&path).join("c_config.toml")).unwrap();
    conf_file
        .write_all(
            format!(
                r#"[general]
target = "{path}"
main = "src/main.c"

[build]
build_dir = "build"
cc = "gcc"
cflags = "-Wall"
"#
            )
            .as_bytes(),
        )
        .unwrap();
    conf_file.flush().unwrap();

    let mut main_file =
        fs::File::create(path::Path::new(&path).join("src").join("main.c")).unwrap();
    main_file
        .write_all(
            r#"#include <stdio.h>

int main(int argc, char **argv) {
    printf("Hello world!\n");
}
"#
            .as_bytes(),
        )
        .unwrap();
    main_file.flush().unwrap();
}
