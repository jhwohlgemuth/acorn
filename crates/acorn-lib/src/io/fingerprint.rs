//! Deterministic content fingerprints for local and remote inputs.
use crate::io::ApiResult;
use crate::prelude::{File, Path, PathBuf, Read, String, ToString, Vec};
use color_eyre::eyre::eyre;
use core::fmt;
use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};

/// SHA-256 fingerprint of one or more identified content values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fingerprint(String);
impl Fingerprint {
    /// Fingerprint an anonymous byte slice.
    pub fn from_bytes(content: impl AsRef<[u8]>) -> Self {
        Self::from_entries([(String::new(), content.as_ref().to_vec())])
    }
    /// Fingerprint identified byte values in deterministic identity order.
    pub fn from_entries<I, K, V>(values: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Vec<u8>>,
    {
        let mut values = values
            .into_iter()
            .map(|(identity, content)| (identity.into(), content.into()))
            .collect::<Vec<_>>();
        values.sort_by(|left, right| left.0.cmp(&right.0));
        let mut context = Context::new(&SHA256);
        values
            .iter()
            .for_each(|(identity, content)| update(&mut context, identity.as_bytes(), content));
        Self(HEXLOWER.encode(context.finish().as_ref()))
    }
    /// Fingerprint a single file using its path as its identity.
    pub fn from_file(path: impl AsRef<Path>) -> ApiResult<Self> {
        Self::from_files([path.as_ref().to_path_buf()])
    }
    /// Fingerprint files in deterministic path order while streaming their contents.
    pub fn from_files<I, P>(paths: I) -> ApiResult<Self>
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        let mut paths = paths.into_iter().map(Into::into).collect::<Vec<_>>();
        paths.sort();
        let mut context = Context::new(&SHA256);
        paths
            .into_iter()
            .try_for_each(|path| fingerprint_file(&mut context, path))
            .map(|()| Self(HEXLOWER.encode(context.finish().as_ref())))
    }
}
impl fmt::Display for Fingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
fn fingerprint_file(context: &mut Context, path: PathBuf) -> ApiResult<()> {
    let identity = path.display().to_string();
    match File::open(&path) {
        | Ok(mut source) => match source.metadata() {
            | Ok(metadata) => {
                update_length(context, identity.len());
                context.update(identity.as_bytes());
                context.update(&metadata.len().to_be_bytes());
                let mut buffer = [0_u8; 8192];
                loop {
                    match source.read(&mut buffer) {
                        | Ok(0) => break Ok(()),
                        | Ok(count) => context.update(buffer.get(..count).unwrap_or_default()),
                        | Err(why) => break Err(eyre!("Read fingerprint source {} — {why}", path.display())),
                    }
                }
            }
            | Err(why) => Err(eyre!("Read fingerprint metadata {} — {why}", path.display())),
        },
        | Err(why) => Err(eyre!("Open fingerprint source {} — {why}", path.display())),
    }
}
fn update(context: &mut Context, identity: &[u8], content: &[u8]) {
    update_length(context, identity.len());
    context.update(identity);
    update_length(context, content.len());
    context.update(content);
}
fn update_length(context: &mut Context, length: usize) {
    context.update(&u64::try_from(length).unwrap_or(u64::MAX).to_be_bytes());
}
