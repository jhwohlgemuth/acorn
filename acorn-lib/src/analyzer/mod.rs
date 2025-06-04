use crate::constants::{APPLICATION, VALE_RELEASES_URL, VALE_VERSION};
use crate::util::{
    download_binary, get_checksum, get_command_folder, get_extension, make_executable, path_to_string, pretty_print, test_command, to_string,
    Constant, Label, ProgrammingLanguage, SemanticVersion,
};
use color_eyre::owo_colors::OwoColorize;
use duct::cmd;
use flate2::read::GzDecoder;
use ini::Ini;
use std::collections::HashMap;
use std::fs::File;
use std::fs::{create_dir_all, remove_file};
use std::io::prelude::*;
use std::path::PathBuf;
use tar::Archive;
use titlecase::Titlecase;
use tracing::{debug, error, info, trace};
use which::which;

pub mod readability;
pub mod vale;

use vale::{parse_vale_output, print_vale_output, Vale, ValeConfig};

pub trait StaticAnalyzer {
    fn command(self) -> String;
    fn analyze(&self, id: String, content: String, output: Option<String>) -> usize;
    fn download(self, config: Option<ValeConfig>) -> Self;
    fn download_checksums(self) -> Result<HashMap<String, String>, String>;
    fn extract(self, path: PathBuf, destination: Option<PathBuf>) -> PathBuf;
    fn sync(self) -> Result<(), std::io::Error>;
    fn with_binary(self, path: PathBuf) -> Self;
    fn with_config(self, value: ValeConfig) -> Self;
    fn with_system_command(self) -> Self;
    fn with_version(self, value: String) -> Self;
}
pub trait StaticAnalyzerConfig {
    fn default() -> ValeConfig;
    fn ini(self) -> Ini;
    fn save(self) -> ValeConfig;
}
impl StaticAnalyzer for Vale {
    fn command(self) -> String {
        "vale".to_string()
    }
    fn analyze(&self, id: String, content: String, output: Option<String>) -> usize {
        let root = get_command_folder("check", None);
        match create_dir_all(root.clone()) {
            | Ok(_) => {}
            | Err(why) => error!(path = path_to_string(root.clone()), "=> {} Create - {}", Label::fail(), why),
        }
        let path = root.join(&id);
        let mut file = match File::create(&path) {
            | Ok(file) => file,
            | Err(why) => panic!("=> {} Create file {} - {}", Label::fail(), path.display(), why),
        };
        file.write_all(content.as_bytes())
            .expect("Unable to write to cache directory project file");
        let binary = match &self.binary {
            | Some(value) => value,
            | None => {
                error!("=> {} {} binary", Label::not_found(), self.clone().command());
                std::process::exit(exitcode::UNAVAILABLE);
            }
        };
        match &self.config {
            | Some(config) => {
                let result = match output {
                    | Some(value) => cmd!(
                        binary,
                        "--no-wrap",
                        "--config",
                        config.clone().path,
                        "--output",
                        value,
                        path.clone(),
                        "--ext",
                        ".md",
                        "--no-exit",
                    )
                    .read(),
                    | None => cmd!(
                        binary,
                        "--no-wrap",
                        "--config",
                        config.clone().path,
                        path.clone(),
                        "--ext",
                        ".md",
                        "--no-exit"
                    )
                    .read(),
                };
                match result {
                    | Ok(output) => {
                        let parsed = parse_vale_output(path, &output);
                        if parsed.is_empty() {
                            let message = format!("=> {} {} has {}", Label::pass(), id.to_string().underline(), "no prose issues".green(),);
                            info!("{}", message);
                            0
                        } else {
                            error!("=> {} {} issues found in {}", Label::fail(), parsed.len(), id.to_string().underline());
                            print_vale_output(parsed.clone());
                            let highlight = parsed.clone().into_iter().map(|item| item.line as usize).collect::<Vec<_>>();
                            println!();
                            pretty_print(&content, ProgrammingLanguage::Markdown, highlight);
                            println!("\n");
                            parsed.len()
                        }
                    }
                    | Err(output) => {
                        error!("=> {} Analyze - {}", Label::fail(), output);
                        1
                    }
                }
            }
            | None => {
                let title = self.clone().command().titlecase();
                error!("=> {} {} configuration", Label::not_found(), title);
                std::process::exit(exitcode::UNAVAILABLE);
            }
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
        let url = format!(
            "{}/download/v{}/{}_{}_{}",
            VALE_RELEASES_URL,
            release,
            self.clone().command(),
            release,
            platform
        );
        info!(url, "=> {} Vale release v{}", Label::using(), release);
        let binary = match download_binary(url.clone(), PathBuf::from(".")) {
            | Ok(path) => {
                let checksum = match self.clone().download_checksums() {
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
                // TODO: Provide option to save to cache project directory
                let destination = match config.clone() {
                    | Some(value) => value.path.parent().unwrap().to_path_buf(),
                    | None => PathBuf::from("./.vale/"),
                };
                let binary = self.clone().extract(path.clone(), Some(destination));
                if make_executable(&binary) {
                    let _cleanup = remove_file(path);
                    Some(binary)
                } else {
                    error!("=> {} {} not executable", Label::fail(), self.command());
                    None
                }
            }
            | Err(error) => {
                error!(error, url, "=> {} {} download", Label::fail(), self.command());
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
        let url = format!(
            "{}/download/v{}/{}_{}_checksums.txt",
            VALE_RELEASES_URL,
            release,
            self.clone().command(),
            release
        );
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
        debug!(
            "=> {} {} checksums {:#?}",
            Label::using(),
            self.command().titlecase(),
            checksums.dimmed().cyan()
        );
        Ok(checksums)
    }
    fn extract(self, path: PathBuf, destination: Option<PathBuf>) -> PathBuf {
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
                let message = format!("Unable to extract {} binary", self.clone().command());
                archive.unpack(parent.clone()).expect(&message);
                debug!(parent, "=> {} Extracted {} binary", Label::using(), self.command());
                PathBuf::from(format!("{}/vale", parent))
            }
        }
    }
    fn sync(self) -> Result<(), std::io::Error> {
        let path = match self.binary {
            | Some(value) => value,
            | None => {
                error!("=> {} {} binary", Label::not_found(), self.command());
                std::process::exit(exitcode::UNAVAILABLE);
            }
        };
        let config_path = self.config.unwrap().path;
        let result = cmd!(path.clone(), "--config", config_path.clone(), "sync").run();
        match result {
            | Ok(_) => {
                let parent = format!("{}/styles/config/vocabularies/{}", config_path.parent().unwrap().display(), APPLICATION);
                debug!(parent, "=> {} Vocabularies", Label::using());
                match create_dir_all(parent.clone()) {
                    | Ok(_) => {}
                    | Err(why) => error!(directory = parent, "=> {} Create - {}", Label::fail(), why),
                }
                match File::create(format!("{}/accept.txt", parent)) {
                    | Ok(mut file) => {
                        // TODO: Concatenate organization alternative names to accept file
                        let acronyms = Constant::get_last_values("acronyms");
                        let partners = Constant::get_last_values("partners");
                        let sponsors = Constant::get_last_values("sponsors");
                        let words = Constant::read_lines("accept.txt");
                        let content = acronyms.chain(partners).chain(sponsors).chain(words).collect::<Vec<String>>().join("\n");
                        file.write_all(content.as_bytes()).expect("Unable to write to accept.txt");
                    }
                    | Err(why) => panic!("=> {} Create accept.txt - {}", Label::fail(), why),
                }
                match File::create(format!("{}/reject.txt", parent)) {
                    | Ok(mut file) => {
                        let content = Constant::read_lines("reject.txt").join("\n");
                        file.write_all(content.as_bytes()).expect("Unable to write to reject.txt");
                    }
                    | Err(why) => panic!("=> {} Create reject.txt - {}", Label::fail(), why),
                }
                Ok(())
            }
            | Err(why) => {
                error!(config = path_to_string(config_path), "=> {} Vale sync - {}", Label::fail(), why);
                std::process::exit(exitcode::SOFTWARE);
            }
        }
    }
    fn with_binary(mut self, path: PathBuf) -> Self {
        self.binary = Some(path);
        self
    }
    fn with_config(mut self, value: ValeConfig) -> Self {
        self.config = Some(value);
        self
    }
    fn with_system_command(mut self) -> Self {
        let name = self.clone().command();
        if test_command(name.clone()) {
            let path = which(name.clone()).unwrap().to_path_buf();
            self.binary = Some(path.clone());
            let offset = "vale version ".len();
            let version = cmd!(name.clone(), "--version").read().unwrap()[offset..].to_string();
            self.version = Some(SemanticVersion::from_string(&version));
            debug!(
                path = path_to_string(path),
                "=> {} System {} (v{}) command",
                Label::using(),
                name.green().bold(),
                version
            );
        }
        self
    }
    fn with_version(mut self, value: String) -> Self {
        self.version = Some(SemanticVersion::from_string(&value));
        self
    }
}
impl StaticAnalyzerConfig for ValeConfig {
    fn default() -> Self {
        let config = ValeConfig::init()
            .packages(to_string(vec!["Google", "proselint", "write-good", "Joblint"]))
            .vocabularies(to_string(vec![APPLICATION]))
            .disabled(to_string(vec![
                "Vale.Terms",
                "Google.EmDash",
                "Google.Contractions",
                "Google.GenderBias",
                "Google.Headings",
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
            .build();
        trace!("=> {} Default - {:#?}", Label::using(), config.dimmed().cyan());
        config
    }
    fn ini(self) -> Ini {
        let ValeConfig {
            packages,
            vocabularies,
            disabled,
            ..
        } = self;
        let mut conf = Ini::new();
        // CAUTION: Order of attributes in INI file matter. "StylesPath" must come before "Vocab"
        conf.with_section::<String>(None)
            .set("StylesPath", "styles")
            .set("Vocab", vocabularies.join(", "))
            .set("Packages", packages.join(", "));
        conf.with_section(Some("*"))
            .set("BasedOnStyles", format!("Vale, {}", packages.join(", ")));
        disabled.iter().for_each(|rule| {
            conf.with_section(Some("*")).set(rule, "NO");
        });
        conf
    }
    fn save(self) -> ValeConfig {
        let path = self.clone().path;
        let parent = path.parent().unwrap().to_path_buf();
        match create_dir_all(parent.clone()) {
            | Ok(_) => {}
            | Err(why) => error!(directory = path_to_string(parent), "=> {} Create - {}", Label::fail(), why),
        }
        match self.clone().ini().write_to_file(path.clone()) {
            | Ok(_) => {
                debug!(path = path_to_string(path), "=> {} Saved configuration", Label::using());
            }
            | Err(why) => {
                error!("=> {} Save configuration - {}", Label::fail(), why);
                std::process::exit(exitcode::SOFTWARE);
            }
        }
        self
    }
}

#[cfg(test)]
mod tests;
