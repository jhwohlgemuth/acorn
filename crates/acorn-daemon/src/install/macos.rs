//! System launchd registration for one selected unprivileged macOS user.
use crate::config::DaemonConfig;
use crate::install::TargetPaths;
use crate::util::constants::app::{LABEL, MANAGED_MARKER, PROPERTY_LIST_PATH};
use acorn_cmd::{args, cmd, sudo};
use directories::BaseDirs;
use serde::Serialize;
use std::env;
use std::fs::{self, Metadata, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct EnvironmentVariables<'a> {
    #[serde(rename = "HOME")]
    home: &'a str,
}
/// Validated user and filesystem inputs used to render a system launchd job.
#[derive(Debug)]
pub(super) struct LaunchIdentity {
    pub(super) user: String,
    pub(super) home: PathBuf,
    pub(super) binary: PathBuf,
    pub(super) config: PathBuf,
}
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct LaunchPropertyList<'a> {
    label: &'static str,
    program_arguments: [&'a str; 3],
    user_name: &'a str,
    environment_variables: EnvironmentVariables<'a>,
    working_directory: &'a str,
    umask: u8,
    run_at_load: bool,
    keep_alive: bool,
}
/// One launchd property list and its filesystem location.
pub(super) struct PropertyList {
    path: PathBuf,
}
impl PropertyList {
    pub(super) fn at(path: &Path) -> Self {
        Self { path: path.to_path_buf() }
    }
    fn installed() -> Self {
        Self::at(Path::new(PROPERTY_LIST_PATH))
    }
    fn path_text(path: &Path) -> io::Result<&str> {
        path.to_str()
            .filter(|value| !value.chars().any(char::is_control))
            .ok_or_else(|| failure("LaunchDaemon paths must be UTF-8 without control characters"))
    }
    /// Render a launchd plist from preflighted paths without filesystem effects.
    pub(super) fn render(identity: &LaunchIdentity) -> io::Result<String> {
        Self::path_text(&identity.binary)
            .and_then(|binary| {
                Self::path_text(&identity.config).and_then(|config| {
                    Self::path_text(&identity.home).and_then(|home| {
                        (!identity.user.chars().any(char::is_control))
                            .then_some(LaunchPropertyList {
                                label: LABEL,
                                program_arguments: [binary, "run", config],
                                user_name: &identity.user,
                                environment_variables: EnvironmentVariables { home },
                                working_directory: home,
                                umask: 63,
                                run_at_load: true,
                                keep_alive: true,
                            })
                            .ok_or_else(|| failure("LaunchDaemon user name contains control characters"))
                    })
                })
            })
            .and_then(|document| {
                let mut bytes = Vec::new();
                plist::to_writer_xml(&mut bytes, &document)
                    .map_err(|error| failure(format!("Cannot serialize LaunchDaemon plist: {error}")))
                    .and_then(|()| String::from_utf8(bytes).map_err(|error| failure(format!("Invalid plist UTF-8: {error}"))))
                    .and_then(|source| {
                        source
                            .split_once("<plist version=\"1.0\">")
                            .map(|(header, body)| format!("{header}<plist version=\"1.0\">\n{MANAGED_MARKER}{body}"))
                            .ok_or_else(|| failure("Serialized plist has no root element"))
                    })
            })
    }
    /// Reject foreign or unsafe plist paths before privileged registration.
    pub(super) fn managed(&self) -> io::Result<bool> {
        match fs::symlink_metadata(&self.path) {
            | Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            | Err(error) => Err(error),
            | Ok(metadata) if !metadata.file_type().is_file() => Err(failure("LaunchDaemon plist path is not a regular file")),
            | Ok(metadata) => fs::read_to_string(&self.path).and_then(|source| {
                plist::Value::from_reader_xml(source.as_bytes())
                    .map_err(|error| failure(format!("Invalid LaunchDaemon plist: {error}")))
                    .and_then(|value| {
                        let label = value
                            .as_dictionary()
                            .and_then(|dictionary| dictionary.get("Label"))
                            .and_then(plist::Value::as_string);
                        let managed = source.contains(MANAGED_MARKER) && Self::root_owned(&metadata) && label == Some(LABEL);
                        managed
                            .then_some(true)
                            .ok_or_else(|| failure("Refusing to replace a foreign or insecure LaunchDaemon plist"))
                    })
            }),
        }
    }
    fn root_owned(metadata: &Metadata) -> bool {
        metadata.uid() == 0 && metadata.gid() == 0 && metadata.permissions().mode() & 0o022 == 0
    }
    fn temporary(source: &str) -> io::Result<Self> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| failure(format!("Cannot name temporary plist: {error}")))
            .and_then(|time| {
                let path = env::temp_dir().join(format!("acornd-{}-{}.plist", std::process::id(), time.as_nanos()));
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&path)
                    .and_then(|mut file| {
                        let written = file.write_all(source.as_bytes()).and_then(|()| file.sync_all());
                        if written.is_err() {
                            let _ = fs::remove_file(&path);
                        }
                        written.map(|()| Self { path })
                    })
            })
    }
    fn lint(&self) -> io::Result<()> {
        cmd!(status "/usr/bin/plutil", args!["-lint", &self.path]).and_then(|status| {
            status
                .success()
                .then_some(())
                .ok_or_else(|| failure(format!("Invalid launchd plist: {status}")))
        })
    }
    fn install_from(&self, source: &Self) -> io::Result<()> {
        Self::path_text(&source.path).and_then(|source| {
            Self::path_text(&self.path)
                .and_then(|destination| sudo!(&["/usr/bin/install", "-o", "root", "-g", "wheel", "-m", "644", source, destination]))
        })
    }
    fn remove(&self) -> io::Result<()> {
        Self::path_text(&self.path).and_then(|path| sudo!(&["/bin/rm", "--", path]))
    }
}
fn failure(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}
/// Register and bootstrap the daemon for the current unprivileged user.
pub fn install(config: &Path) -> io::Result<()> {
    let destination = PropertyList::installed();
    preflight(config)
        .and_then(|identity| PropertyList::render(&identity))
        .and_then(|source| PropertyList::temporary(&source))
        .and_then(|temporary| {
            let result = temporary
                .lint()
                .and_then(|()| destination.managed())
                .and_then(|present| {
                    let unload = if present { unload_if_loaded() } else { Ok(()) };
                    unload.and_then(|()| destination.install_from(&temporary))
                })
                .and_then(|()| destination.managed())
                .and_then(|managed| managed.then_some(()).ok_or_else(|| failure("Installed plist was not ACORN-managed")))
                .and_then(|()| sudo!(&["/bin/launchctl", "bootstrap", "system", PROPERTY_LIST_PATH]));
            fs::remove_file(&temporary.path).and(result)
        })
}
pub(super) fn launchd_presence(code: Option<i32>, stderr: &[u8]) -> io::Result<bool> {
    match code {
        | Some(0) => Ok(true),
        | Some(113) => Ok(false),
        | _ => Err(failure(format!(
            "Cannot inspect system LaunchDaemon: {}",
            String::from_utf8_lossy(stderr).chars().take(500).collect::<String>()
        ))),
    }
}
fn owner_home(path: &Path, uid: u32) -> io::Result<()> {
    fs::symlink_metadata(path).and_then(|metadata| {
        (metadata.file_type().is_dir() && metadata.uid() == uid && metadata.permissions().mode() & 0o022 == 0)
            .then_some(())
            .ok_or_else(|| failure(format!("Expected a trusted user home: {}", path.display())))
    })
}
/// Validate an unprivileged owner and the paired daemon inputs before registration.
fn preflight(config: &Path) -> io::Result<LaunchIdentity> {
    text_command("/usr/bin/id", &["-u"])
        .and_then(|uid| {
            uid.parse::<u32>()
                .map_err(|_| failure("Cannot determine the current user ID"))
                .and_then(|uid| {
                    (uid != 0)
                        .then_some(uid)
                        .ok_or_else(|| failure("Run installation as the selected user, not root"))
                })
        })
        .and_then(|uid| {
            text_command("/usr/bin/id", &["-un"]).and_then(|user| {
                BaseDirs::new()
                    .map(|dirs| dirs.home_dir().to_path_buf())
                    .ok_or_else(|| failure("Cannot resolve the selected user's home directory"))
                    .and_then(|home| owner_home(&home, uid).map(|()| home))
                    .and_then(|home| {
                        TargetPaths::resolve()
                            .map_err(|error| failure(error.to_string()))
                            .and_then(|targets| {
                                private_regular(&targets.data().join("token"), uid)
                                    .and_then(|()| fs::read_to_string(targets.data().join("token")))
                                    .and_then(|token| {
                                        (!token.trim().is_empty() && !token.trim().chars().any(char::is_control))
                                            .then_some(())
                                            .ok_or_else(|| failure("Daemon token is empty or malformed"))
                                    })
                            })
                            .and_then(|()| private_regular(config, uid))
                            .and_then(|()| fs::canonicalize(config))
                            .and_then(|config| {
                                DaemonConfig::load(&config)
                                    .map_err(|error| failure(format!("Invalid daemon configuration: {error}")))
                                    .map(|_| config)
                            })
                            .and_then(|config| {
                                env::current_exe()
                                    .and_then(fs::canonicalize)
                                    .map(|binary| LaunchIdentity { user, home, binary, config })
                            })
                    })
            })
        })
}
pub(super) fn private_regular(path: &Path, uid: u32) -> io::Result<()> {
    fs::symlink_metadata(path).and_then(|metadata| {
        (metadata.file_type().is_file() && metadata.uid() == uid && metadata.permissions().mode() & 0o077 == 0)
            .then_some(())
            .ok_or_else(|| failure(format!("Expected an owner-only regular file: {}", path.display())))
    })
}
/// Restart the registered system LaunchDaemon.
pub fn restart() -> io::Result<()> {
    stop().and_then(|()| start())
}
/// Start the registered system LaunchDaemon.
pub fn start() -> io::Result<()> {
    let target = format!("system/{LABEL}");
    PropertyList::installed()
        .managed()
        .and_then(|managed| managed.then_some(()).ok_or_else(|| failure("ACORN LaunchDaemon is not installed")))
        .and_then(|()| cmd!("/bin/launchctl", args!["print", &target]))
        .and_then(|output| launchd_presence(output.status.code(), &output.stderr))
        .and_then(|loaded| {
            if loaded {
                sudo!(&["/bin/launchctl", "kickstart", &target])
            } else {
                sudo!(&["/bin/launchctl", "bootstrap", "system", PROPERTY_LIST_PATH])
            }
        })
}
/// Show launchd status for the registered daemon.
pub fn status() -> io::Result<()> {
    PropertyList::installed()
        .managed()
        .and_then(|managed| managed.then_some(()).ok_or_else(|| failure("ACORN LaunchDaemon is not installed")))
        .and_then(|()| {
            let target = format!("system/{LABEL}");
            cmd!(status "/bin/launchctl", args!["print", &target]).and_then(|status| {
                status
                    .success()
                    .then_some(())
                    .ok_or_else(|| failure(format!("launchctl print failed: {status}")))
            })
        })
}
/// Stop and unload the registered system LaunchDaemon.
pub fn stop() -> io::Result<()> {
    PropertyList::installed()
        .managed()
        .and_then(|managed| managed.then_some(()).ok_or_else(|| failure("ACORN LaunchDaemon is not installed")))
        .and_then(|()| unload_if_loaded())
}
fn text_command(program: &str, arguments: &[&str]) -> io::Result<String> {
    cmd!(program, args![..arguments.iter().copied()]).and_then(|output| {
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .filter(|text| !text.is_empty())
            .ok_or_else(|| failure(format!("{program} failed")))
    })
}
/// Stop and remove only the ACORN-managed LaunchDaemon plist.
pub fn uninstall() -> io::Result<()> {
    let plist = PropertyList::installed();
    plist.managed().and_then(|managed| match managed {
        | true => unload_if_loaded().and_then(|()| plist.remove()),
        | false => Ok(()),
    })
}
fn unload_if_loaded() -> io::Result<()> {
    let target = format!("system/{LABEL}");
    cmd!("/bin/launchctl", args!["print", &target])
        .and_then(|output| launchd_presence(output.status.code(), &output.stderr))
        .and_then(|loaded| {
            if loaded {
                sudo!(&["/bin/launchctl", "bootout", &target])
            } else {
                Ok(())
            }
        })
}
