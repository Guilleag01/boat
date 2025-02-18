use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: General,
    pub build: Build,
}

#[derive(Debug, Deserialize)]
pub struct General {
    pub target: String,
    pub main: String,
}

#[derive(Debug, Deserialize)]
pub struct Build {
    pub build_dir: String,
    pub cc: String,
    pub cflags: String,
}
