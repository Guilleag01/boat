use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;
use std::{collections::HashSet, fs, path};

use crate::config::Config;

pub struct Compiler {
    config: Config,
    path: String,
    src_files: Vec<String>,
    header_files: Vec<String>,
    needed_src: Vec<String>,
    inc_dirs: Vec<String>,
}

impl Compiler {
    pub fn new(
        config: Config,
        path: String,
        src_files: Vec<String>,
        header_files: Vec<String>,
    ) -> Self {
        Self {
            config,
            path,
            src_files,
            header_files,
            needed_src: Vec::new(),
            inc_dirs: Vec::new(),
        }
    }

    pub fn prepare(&mut self) {
        let (needed_src, inc_dirs) = self.get_needed_files();
        self.needed_src = needed_src;
        self.inc_dirs = inc_dirs;
    }

    pub fn compile(&self, verbose: bool) {
        let mut o_s: Vec<String> = Vec::new();

        for file in self.needed_src.clone() {
            let o_file = format!("{}.o", file.split(".").collect::<Vec<&str>>()[0]);
            let o_path = path::Path::new(&o_file);
            let mut o_comp = o_path.components();
            o_comp.next();

            let o_path_dir = path::Path::new(&self.path)
                .join(&self.config.build.build_dir)
                .join(o_comp.as_path());

            let o_path_strip = o_path_dir.to_str().unwrap();

            println!("Compiling {}", file);

            let mut c_path = path::Path::new(&file).parent().unwrap().components();
            c_path.next();

            let c_path_full = path::Path::new(&self.path)
                .join(self.config.build.build_dir.clone())
                .join(c_path.as_path());

            let mkdir_command = format!("mkdir -p {}", c_path_full.to_str().unwrap());

            Command::new("sh")
                .arg("-c")
                .arg(mkdir_command)
                .output()
                .unwrap();

            // TODO: PRINT WHEN VERBOSE

            let command = format!(
                "{} {} -c {} {} -o {}",
                self.config.build.cc,
                self.config.build.cflags,
                file.clone(),
                self.get_inc_string(),
                o_path_strip
            );

            if verbose {
                println!("{}", command);
            }

            o_s.push(o_path_strip.to_string());

            let out = Command::new("sh").arg("-c").arg(command).output().unwrap();

            let stdout = std::str::from_utf8(&out.stdout).unwrap();
            let stderr = std::str::from_utf8(&out.stderr).unwrap();

            if !stdout.is_empty() {
                println!("{}", stdout);
            }
            if !stderr.is_empty() {
                println!("{}", stderr);
            }
        }

        let mut link_command = format!(
            "{} {}",
            self.config.build.cc.clone(),
            self.config.build.cflags
        );

        for o_file in o_s {
            link_command = format!("{} {}", link_command, o_file);
        }

        let target_path = path::Path::new(&self.path).join(&self.config.general.target);

        link_command = format!(
            "{} {} -o {}",
            link_command,
            self.get_inc_string(),
            target_path.to_str().unwrap()
        );

        println!("Building {}", target_path.to_str().unwrap());

        if verbose {
            println!("{}", link_command);
        }

        let out = Command::new("sh")
            .arg("-c")
            .arg(link_command)
            .output()
            .unwrap();

        let stdout = std::str::from_utf8(&out.stdout).unwrap();
        let stderr = std::str::from_utf8(&out.stderr).unwrap();

        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            println!("{}", stderr);
        }
    }

    pub fn get_needed_files(&self) -> (Vec<String>, Vec<String>) {
        let mut scanned_files = Vec::new();

        let (mut src_file, header_files) = self.get_needed_files_recursive(
            path::Path::new(self.path.as_str())
                .join(self.config.general.main.clone())
                .to_str()
                .unwrap()
                .to_string(),
            &mut scanned_files,
        );
        src_file.push(
            path::Path::new(self.path.as_str())
                .join(self.config.general.main.clone())
                .to_str()
                .unwrap()
                .to_string(),
        );
        (src_file, header_files)
    }

    fn get_needed_files_recursive(
        &self,
        file: String,
        scanned_files: &mut Vec<String>,
    ) -> (Vec<String>, Vec<String>) {
        let file_path = path::Path::new(file.as_str());

        let contents = fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Couldn't read {}", file_path.to_str().unwrap()));

        lazy_static! {
            static ref RE: Regex = Regex::new(r#"#include (<\w+.h>|\"\w+\.h\")"#).unwrap();
        }

        let mut header_paths: HashSet<String> = HashSet::new();
        let mut src_paths: HashSet<String> = HashSet::new();

        let headers_names = self
            .header_files
            .iter()
            .map(|name| {
                path::Path::new(name.as_str())
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<String>>();

        let src_names = self
            .src_files
            .iter()
            .map(|name| {
                path::Path::new(name.as_str())
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<String>>();

        let finds = RE
            .find_iter(&contents)
            .map(|f| f.as_str()[10..f.len() - 1].to_string())
            .collect::<Vec<String>>();

        // let mut scanned_files = Vec::new();

        for find in finds.clone() {
            // let find_name = fin[10..find.len() - 1].to_string();

            if headers_names.contains(&find) {
                let h_path = path::Path::new(
                    self.header_files[headers_names.iter().position(|e| *e == find).unwrap()]
                        .as_str(),
                );

                let h_dir = h_path.parent().unwrap().to_str().unwrap().to_string();

                header_paths.insert(h_dir.clone());

                if !scanned_files.contains(&h_path.to_str().unwrap().to_string()) {
                    scanned_files.push(h_path.to_str().unwrap().to_string());
                    let (recursive_src, recursive_header) = self.get_needed_files_recursive(
                        h_path.to_str().unwrap().to_string(),
                        scanned_files,
                    );

                    for e in recursive_src {
                        src_paths.insert(e);
                    }
                    for e in recursive_header {
                        header_paths.insert(e);
                    }
                }
            }

            let h_to_c = format!("{}.c", find.split(".").collect::<Vec<&str>>()[0]);

            if src_names.contains(&h_to_c) {
                let c_path =
                    self.src_files[src_names.iter().position(|e| *e == h_to_c).unwrap()].clone();

                src_paths.insert(c_path.clone());

                if !scanned_files.contains(&c_path) {
                    scanned_files.push(c_path.clone());

                    let (recursive_src, recursive_header) =
                        self.get_needed_files_recursive(c_path.clone(), scanned_files);

                    for e in recursive_src {
                        src_paths.insert(e);
                    }
                    for e in recursive_header {
                        header_paths.insert(e);
                    }
                }
            }
        }

        // println!("finds: \n{:?}", src_names);

        (
            src_paths.iter().cloned().collect::<Vec<String>>(),
            header_paths.iter().cloned().collect::<Vec<String>>(),
        )
    }

    pub fn clean(&self) {
        let build_dir = path::Path::new(&self.path).join(self.config.build.build_dir.clone());
        let target_path = path::Path::new(&self.path).join(self.config.general.target.clone());
        println!("Removing dir {}", build_dir.to_str().unwrap());
        fs::remove_dir_all(build_dir).unwrap();
        println!("Removing file {}", target_path.to_str().unwrap());
        fs::remove_file(target_path).unwrap();
    }

    pub fn run(&self) {
        let target_path = path::Path::new(&self.path).join(self.config.general.target.clone());
        println!("Running {}", target_path.to_str().unwrap());
        Command::new("sh")
            .arg("-c")
            .arg(format!("./{}", target_path.to_str().unwrap()).as_str())
            .output()
            .unwrap();
    }

    fn get_inc_string(&self) -> String {
        let mut inc_string = "".to_string();

        for inc_dir in self.inc_dirs.clone() {
            inc_string = format!("{} -I{}", inc_string, inc_dir);
        }

        inc_string
    }
}
