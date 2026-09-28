//! Staged multi-file workflow application with in-process rollback.
use super::Artifact;
use crate::io::ApiResult;
use crate::util::constants::app::APPLICATION;
use acorn_core::prelude::Vec;
use color_eyre::eyre::eyre;
use nanoid::nanoid;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) struct Artifacts(Vec<Artifact>);
pub(super) struct StagedFile {
    pub(super) backup: Option<PathBuf>,
    pub(super) stage: PathBuf,
    pub(super) target: PathBuf,
}
pub(super) struct StagedFiles(Vec<StagedFile>);
impl Artifact {
    fn stage(self) -> ApiResult<StagedFile> {
        let Self { path: target, content } = self;
        let stage = sibling(&target, "stage");
        let backup = target.exists().then(|| sibling(&target, "backup"));
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&stage)
            .and_then(|mut output| output.write_all(content.as_bytes()).and_then(|()| output.sync_all()))
            .map_err(|why| eyre!("failed to stage workflow file `{}` — {why}", target.display()))
            .map(|()| StagedFile { backup, stage, target })
    }
}
impl Artifacts {
    pub(crate) fn apply(self) -> ApiResult<usize> {
        self.normalize().and_then(Self::stage).and_then(StagedFiles::commit)
    }
    fn normalize(self) -> ApiResult<Self> {
        self.0
            .into_iter()
            .try_fold(Vec::<Artifact>::new(), |mut targets, file| {
                let is_valid_parent = file.path.parent().is_some_and(|parent| {
                    parent
                        .symlink_metadata()
                        .ok()
                        .is_some_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
                });
                let is_valid_target = file
                    .path
                    .symlink_metadata()
                    .map_or(true, |metadata| metadata.is_file() && !metadata.file_type().is_symlink());
                match targets.iter().find(|target| target.path == file.path) {
                    | _ if !is_valid_parent => Err(eyre!("workflow destination parent is unavailable or unsafe: `{}`", file.path.display())),
                    | _ if !is_valid_target => Err(eyre!("workflow destination is unavailable or unsafe: `{}`", file.path.display())),
                    | Some(target) if target.content == file.content => Ok(targets),
                    | Some(_) => Err(eyre!("conflicting workflow destination `{}`", file.path.display())),
                    | None => {
                        targets.push(file);
                        Ok(targets)
                    }
                }
            })
            .map(Self)
    }
    fn stage(self) -> ApiResult<StagedFiles> {
        self.0
            .into_iter()
            .try_fold(Vec::new(), |mut staged, file| match file.stage() {
                | Ok(next) => {
                    staged.push(next);
                    Ok(staged)
                }
                | Err(why) => {
                    staged.iter().for_each(|file| {
                        fs::remove_file(&file.stage).ok();
                    });
                    Err(why)
                }
            })
            .map(StagedFiles)
    }
}
impl From<Vec<Artifact>> for Artifacts {
    fn from(files: Vec<Artifact>) -> Self {
        Self(files)
    }
}
impl StagedFile {
    fn prepare_backup(&self) -> ApiResult<()> {
        match &self.backup {
            | Some(backup) => {
                fs::rename(&self.target, backup).map_err(|why| eyre!("failed to prepare workflow backup `{}` — {why}", self.target.display()))
            }
            | None => Ok(()),
        }
    }
}
impl From<Vec<StagedFile>> for StagedFiles {
    fn from(files: Vec<StagedFile>) -> Self {
        Self(files)
    }
}
impl StagedFiles {
    pub(super) fn commit(self) -> ApiResult<usize> {
        self.0
            .iter()
            .try_fold(0_usize, |committed, file| {
                file.prepare_backup().and_then(|()| {
                    fs::rename(&file.stage, &file.target)
                        .map_err(|why| eyre!("failed to commit workflow file `{}` — {why}", file.target.display()))
                        .map(|()| committed.saturating_add(1))
                })
            })
            .inspect(|_| {
                self.0.iter().for_each(|file| {
                    file.backup.as_ref().and_then(|backup| fs::remove_file(backup).ok());
                });
            })
            .inspect_err(|_| self.rollback())
    }
    fn rollback(&self) {
        self.0.iter().rev().for_each(|file| {
            if file.backup.as_ref().is_some_and(|backup| backup.exists()) {
                fs::remove_file(&file.target).ok();
                if let Some(backup) = &file.backup {
                    fs::rename(backup, &file.target).ok();
                }
            } else if !file.stage.exists() {
                fs::remove_file(&file.target).ok();
            }
            fs::remove_file(&file.stage).ok();
        })
    }
}
fn sibling(path: &Path, kind: &str) -> PathBuf {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!(".{name}.{APPLICATION}-{kind}-{}", nanoid!(10)))
}
