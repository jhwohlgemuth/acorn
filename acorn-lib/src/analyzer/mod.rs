use crate::util::*;
use bon::builder;
use color_eyre::owo_colors::OwoColorize;
use derive_more::Display;
use directories::ProjectDirs;
use duct::cmd;
use flate2::read::GzDecoder;
use ini::Ini;
use std::collections::HashMap;
use std::fs::remove_file;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use tar::Archive;
use tracing::{debug, error, info};

const VALE_RELEASES_URL: &str = "https://github.com/errata-ai/vale/releases";
const VALE_VERSION: &str = "3.9.4";

pub trait StaticAnalyzer {
    fn analyze(self, id: String, content: String) -> Result<std::process::Output, std::io::Error>;
    fn download(self, config: Option<ValeConfig>) -> Self;
    fn download_checksums(self) -> Result<HashMap<String, String>, String>;
    fn extract(path: PathBuf, destination: Option<PathBuf>) -> PathBuf;
    fn sync(self) -> Result<std::process::Output, std::io::Error>;
}
#[derive(Clone, Debug, Default, bon::Builder, Display)]
#[display("{:?}", version)]
#[builder(start_fn = init)]
pub struct Vale {
    pub version: Option<SemanticVersion>,
    pub binary: Option<PathBuf>,
    pub config: Option<ValeConfig>,
}
#[derive(Clone, Debug, Display, bon::Builder)]
#[display("{:?}", path)]
#[builder(start_fn = init)]
pub struct ValeConfig {
    #[builder(default = PathBuf::from("./.vale/.vale.ini"))]
    pub path: PathBuf,
    #[builder(default = Vec::<String>::new())]
    pub packages: Vec<String>,
    #[builder(default = Vec::<String>::new())]
    pub vocabularies: Vec<String>,
    #[builder(default = Vec::<String>::new())]
    pub disabled: Vec<String>,
}
impl StaticAnalyzer for Vale {
    fn analyze(self, id: String, content: String) -> Result<std::process::Output, std::io::Error> {
        match ProjectDirs::from("org", "ornl", "acorn") {
            | Some(dirs) => {
                let cache = dirs.cache_dir();
                let path = cache.join(format!("{}.txt", id));
                std::fs::create_dir_all(cache).expect("Unable to create cache directory {cache}");
                let mut file = match File::create(&path) {
                    | Ok(file) => file,
                    | Err(why) => panic!("=> {} Create file {} - {}", Label::fail(), path.display(), why),
                };
                // std::fs::write(path, content).expect("Unable to write to temporary file");
                file.write_all(content.as_bytes()).expect("Unable to write to temporary file");
                let binary = match self.binary {
                    | Some(value) => value,
                    | None => {
                        error!("=> {} Vale binary", Label::not_found());
                        std::process::exit(exitcode::UNAVAILABLE);
                    }
                };
                println!("\n\n");
                error!("=> {} {} = {}", Label::using(), id.underline(), content);
                match self.config {
                    | Some(config) => cmd!(binary, "--config", config.path, "--no-wrap", path).run(),
                    | None => {
                        error!("=> {} Vale configuration", Label::not_found());
                        std::process::exit(exitcode::UNAVAILABLE);
                    }
                }
            }
            | None => unreachable!(),
        }
    }
    // TODO: Check if binary has already been downloaded
    fn download(self, config: Option<ValeConfig>) -> Vale {
        // https://doc.rust-lang.org/std/env/consts/constant.OS.html
        let os = std::env::consts::OS.to_lowercase();
        let platform = match os.as_str() {
            | "linux" => "Linux_64-bit.tar.gz",
            | "macos" | "apple" => "macOS_64-bit.tar.gz",
            | "windows" => "Windows_64-bit.zip",
            | _ => {
                error!(os, "=> {}", Label::not_found());
                std::process::exit(exitcode::UNAVAILABLE);
            }
        };
        let release = match self.version {
            | Some(value) => value,
            | None => SemanticVersion::from_string(VALE_VERSION),
        };
        let url = format!("{}/download/v{}/vale_{}_{}", VALE_RELEASES_URL, release, release, platform);
        info!(url, "=> {} Vale release v{}", Label::using(), release);
        let binary = match download_binary(url.clone(), PathBuf::from(".")) {
            | Ok(path) => {
                let checksum = match self.download_checksums() {
                    | Ok(value) => value.get(platform).unwrap().to_string(),
                    | Err(_) => "".to_string(),
                };
                let calculated = get_checksum(path.clone());
                if !checksum.eq(&calculated) {
                    error!(checksum, calculated, "=> {}", Label::invalid());
                    let _cleanup = remove_file(path);
                    std::process::exit(exitcode::USAGE);
                } else {
                    info!(checksum, "=> {}", Label::pass());
                }
                let destination = match config.clone() {
                    | Some(value) => value.path.parent().unwrap().to_path_buf(),
                    | None => PathBuf::from("./.vale/"),
                };
                let binary = Vale::extract(path.clone(), Some(destination));
                if make_executable(&binary) {
                    let _cleanup = remove_file(path);
                    Some(binary)
                } else {
                    error!("=> {} vale not executable", Label::fail());
                    None
                }
            }
            | Err(error) => {
                error!(error, url, "=> {} vale download", Label::fail());
                None
            }
        };
        let builder = Vale::init().version(release).maybe_binary(binary);
        match config {
            | Some(value) => builder.config(value).build(),
            | None => {
                let config = ValeConfig::default();
                builder.config(config).build()
            }
        }
    }
    fn download_checksums(self) -> Result<HashMap<String, String>, String> {
        let release = match self.version {
            | Some(value) => value,
            | None => SemanticVersion::from_string(VALE_VERSION),
        };
        let url = format!("{}/download/v{}/vale_{}_checksums.txt", VALE_RELEASES_URL, release, release);
        let client = reqwest::blocking::Client::new();
        let response = client.get(url).send().unwrap();
        let content = response.text().unwrap();
        let checksums = content.lines().clone().fold(HashMap::new(), |mut acc: HashMap<String, String>, line| {
            let mut values = line.split("  ").collect::<Vec<&str>>();
            let key = values.pop().unwrap()["vale_#.#.#_".len()..].to_string();
            let value = values.pop().unwrap().to_string();
            acc.insert(key, value);
            acc
        });
        debug!("=> {} Vale checksums {:#?}", Label::using(), checksums);
        Ok(checksums)
    }
    fn extract(path: PathBuf, destination: Option<PathBuf>) -> PathBuf {
        match get_extension(&path).as_str() {
            | "zip" => unimplemented!(),
            | _ => {
                let tar_gz = File::open(path).unwrap();
                let tar = GzDecoder::new(tar_gz);
                let mut archive = Archive::new(tar);
                let parent = match destination {
                    | Some(value) => path_to_string(value),
                    | None => "./.vale/".to_string(),
                };
                archive.unpack(parent.clone()).expect("Unable to extract vale binary");
                debug!(parent, "=> {} Extracted vale binary", Label::using());
                PathBuf::from(format!("{}/vale", parent))
            }
        }
    }
    fn sync(self) -> Result<std::process::Output, std::io::Error> {
        let path = match self.binary {
            | Some(value) => value,
            | None => {
                error!("=> {} Vale binary", Label::not_found());
                std::process::exit(exitcode::UNAVAILABLE);
            }
        };
        cmd!(path.clone(), "--config", self.config.unwrap().path, "sync").run()
    }
}
impl Default for ValeConfig {
    fn default() -> Self {
        debug!("=> {} Default Vale configuration", Label::using());
        ValeConfig::init()
            .packages(to_string(vec!["Google", "proselint", "write-good", "Readability", "Joblint"]))
            .vocabularies(to_string(vec![]))
            .disabled(to_string(vec![
                "Vale.Terms",
                "Google.EmDash",
                "Google.Contractions",
                "Google.GenderBias",
                "Google.Parens",
                "Google.Quotes",
                "Google.We",
                "Joblint.Competitive",
                "proselint.GenderBias",
                "write-good.E-Prime",
                "write-good.Passive",
                "write-good.TooWordy",
                "write-good.Weasel",
            ]))
            .build()
    }
}
impl ValeConfig {
    pub fn ini(&self) -> Ini {
        let ValeConfig {
            packages,
            vocabularies,
            disabled,
            ..
        } = self;
        let mut conf = Ini::new();
        conf.with_section::<String>(None)
            .set("Packages", packages.join(", "))
            .set("Vocab", vocabularies.join(", "))
            .set("StylesPath", "styles");
        conf.with_section(Some("*"))
            .set("BasedOnStyles", format!("Vale, {}", packages.join(", ")));
        disabled.iter().for_each(|rule| {
            conf.with_section(Some("*")).set(rule, "NO");
        });
        conf
    }
    pub fn save(self) -> ValeConfig {
        match self.ini().write_to_file(self.clone().path) {
            | Ok(_) => (),
            | Err(why) => {
                error!("=> {} Save configuration - {}", Label::fail(), why);
            }
        }
        self
    }
}
