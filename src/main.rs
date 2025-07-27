use std::{
    fmt::Display,
    fs,
    path::{self, Path},
};

use boat::{compiler::Compiler, config::Config, init};
use clap::Parser;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Modes {
    Build,
    Clean,
    Run,
    Init,
}

impl Display for Modes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Modes::Build => f.write_str("build"), // "build".to_string(),
            Modes::Clean => f.write_str("clean"),
            Modes::Run => f.write_str("run"),
            Modes::Init => f.write_str("init"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Verbose
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Mode
    #[arg(  default_value_t = Modes::Build)]
    mode: Modes,

    /// Path of the directory to list
    #[arg(default_value_t = String::from("."))]
    path: String,
}

fn main() {
    let args = Args::parse();

    let mut compiler = Compiler::default();

    if let Modes::Init = args.mode {
    } else {
        let conf_path = path::Path::new(args.path.as_str()).join("c_config.toml");

        let contents = if let Ok(s) = fs::read_to_string(conf_path.clone()) {
            s
        } else {
            eprintln!("Error: Couldn't read c_config.toml");
            return;
        };

        let config: Config = toml::from_str(&contents).unwrap();

        let (src_files, header_files) =
            get_file_list(&args.path).expect("Error while readig files");

        compiler = Compiler::new(config, args.path.clone(), src_files, header_files);
    }

    match args.mode {
        Modes::Build => {
            compiler.prepare();
            compiler.compile(args.verbose);
        }
        Modes::Clean => compiler.clean(),
        Modes::Run => {
            compiler.prepare();
            compiler.compile(args.verbose);
            compiler.run();
        }
        Modes::Init => {
            init::init(args.path);
        }
    }
}

fn get_file_list(path_str: &str) -> Result<(Vec<String>, Vec<String>), std::io::Error> {
    let path = Path::new(path_str);
    let mut src_files = Vec::new();
    let mut header_files = Vec::new();

    if path.is_dir() {
        for entry in std::fs::read_dir(path_str)? {
            let entry = entry?;

            if entry.path().is_dir() {
                let (mut r_src_files, mut r_header_files) =
                    get_file_list(entry.path().to_str().unwrap())?;

                header_files.append(&mut r_header_files);
                src_files.append(&mut r_src_files);
            }

            let f_name = entry.path().to_str().unwrap().to_string();

            match f_name.as_str().split(".").last().unwrap() {
                "h" => header_files.push(f_name),
                "c" => src_files.push(f_name),
                _ => (),
            }

            // files.push();
        }
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "The path is not a valid directory.",
        ));
    }

    Ok((src_files, header_files))
}
