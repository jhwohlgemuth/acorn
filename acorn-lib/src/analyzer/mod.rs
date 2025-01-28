use crate::util::*;
use bon::builder;
use derive_more::Display;
use duct::cmd;
use flate2::read::GzDecoder;
use ini::Ini;
use std::collections::HashMap;
use std::fs::remove_file;
use std::fs::File;
use std::path::PathBuf;
use tar::Archive;
use tracing::{debug, error, info};

const VALE_RELEASES_URL: &str = "https://github.com/errata-ai/vale/releases";
const VALE_VERSION: &str = "3.9.4";

pub trait StaticAnalyzer {
    fn download(self) -> Result<PathBuf, String>;
    fn download_checksums(self) -> Result<HashMap<String, String>, String>;
    fn extract(path: PathBuf, destination: Option<PathBuf>) -> PathBuf;
    fn run(self, binary_location: PathBuf) -> Result<std::process::Output, std::io::Error>;
}
#[derive(Clone, Debug, bon::Builder, Display)]
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
    fn download(self) -> Result<PathBuf, String> {
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
        let release = match self.version.clone() {
            | Some(value) => value,
            | None => SemanticVersion::from_string(VALE_VERSION),
        };
        let url = format!("{}/download/v{}/vale_{}_{}", VALE_RELEASES_URL, release, release, platform);
        let destination = PathBuf::from(".");
        info!(url, "=> {} Vale release v{}", Label::using(), release);
        match download_binary(url.clone(), destination) {
            | Ok(path) => {
                let checksum = match self.download_checksums() {
                    | Ok(value) => value.get(platform).unwrap().to_string(),
                    | Err(_) => "".to_string(),
                };
                let expected = get_checksum(path.clone());
                if !checksum.eq(&expected) {
                    error!(checksum, expected, "=> {}", Label::invalid());
                    let _cleanup = remove_file(path);
                    std::process::exit(exitcode::USAGE);
                } else {
                    info!(checksum, "=> {}", Label::pass());
                }
                let binary = Vale::extract(path.clone(), None);
                if make_executable(&binary) {
                    let _cleanup = remove_file(path);
                    Ok(binary)
                } else {
                    error!("=> {} vale not executable", Label::fail());
                    Err(format!("Failed to make {} executable", binary.display()))
                }
            }
            | Err(error) => {
                error!(error, url, "=> {} Vale download", Label::fail());
                Err(error)
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
                let _ = archive.unpack(parent.clone());
                debug!(parent, "=> {} Extracted vale binary", Label::using());
                PathBuf::from(format!("{}/vale", parent))
            }
        }
    }
    fn run(self, binary_location: PathBuf) -> Result<std::process::Output, std::io::Error> {
        let vale = path_to_string(binary_location);
        let _config = ValeConfig::default().ini();
        cmd!("echo", "hellow world")
            .pipe(cmd!(vale.clone(), "--config", "./.vale/.vale.ini", "sync"))
            .read()
            .unwrap();
        // cmd!(vale, "--config", "./.vale/.vale.ini", "sync").run()
        cmd!(vale, "--version").run()
    }
}
impl ValeConfig {
    pub fn default() -> ValeConfig {
        ValeConfig::init()
            .packages(to_string(vec!["Google", "proselint", "write-good", "Readability", "Joblint"]))
            .vocabularies(to_string(vec!["ORNL"]))
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
    pub fn ini(self) -> Ini {
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
        disabled.into_iter().for_each(|rule| {
            conf.with_section(Some("*")).set(rule, "NO");
        });
        conf.write_to(&mut std::io::stdout()).unwrap();
        let _save = conf.write_to_file(self.path);
        conf
    }
}
