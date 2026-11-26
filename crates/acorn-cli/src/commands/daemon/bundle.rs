//! Verification of signed paired daemon bundles before installation.

use alloc::collections::BTreeMap;
use color_eyre::{eyre::eyre, Result};
use data_encoding::HEXLOWER;
use flate2::read::GzDecoder;
use minisign_verify::{PublicKey, Signature};
use ring::digest::{Context, SHA256};
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
#[cfg(all(unix, any(target_os = "linux", test)))]
use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};
use tar::Archive;

const PUBLIC_KEY: &str = include_str!("../../../assets/acornd-release.pub");
const MAX_BUNDLE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_BINARY_BYTES: u64 = 256 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 16 * 1024;
const MAX_SIGNATURE_BYTES: u64 = 16 * 1024;
const FORMAT_VERSION: u32 = 1;
const TARGETS: [&str; 5] = [
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-musl",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-musl",
];

/// Signed bundle manifest format v1, stored as `manifest.json` in a gzip tar archive.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct BundleManifest {
    format_version: u32,
    version: String,
    target: String,
    binaries: Vec<BinaryManifest>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct BinaryManifest {
    name: String,
    sha256: String,
    size: u64,
}

#[derive(Default)]
struct Contents {
    manifest: Option<BundleManifest>,
    binaries: BTreeMap<String, (u64, String)>,
}

/// Verify a detached Minisign signature and every member of a paired release bundle.
///
/// Callers must pass the same private staging file handle to subsequent extraction;
/// the verified archive must not be replaced between verification and activation.
pub(crate) fn verify_bundle<R: Read + Seek>(
    archive: &mut R,
    signature: &str,
    expected_version: &str,
    expected_target: &str,
) -> Result<BundleManifest> {
    PublicKey::decode(PUBLIC_KEY)
        .map_err(|error| eyre!("Pinned bundle public key is invalid: {error}"))
        .and_then(|key| verify_with_key(archive, signature, expected_version, expected_target, &key))
}

/// Verify a release archive and detached signature at their filesystem paths.
pub(crate) fn verify_path(path: &Path) -> Result<()> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| eyre!("Bundle has no UTF-8 filename"));
    name.and_then(|name| {
        TARGETS
            .iter()
            .find_map(|target| {
                name.strip_prefix("acornd-v")
                    .and_then(|remainder| remainder.strip_suffix(&format!("-{target}.tar.gz")))
                    .map(|version| (version.to_string(), *target))
            })
            .filter(|(version, _)| {
                version.split('.').count() == 3
                    && version
                        .split('.')
                        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
            })
            .ok_or_else(|| eyre!("Bundle filename does not name a supported release"))
    })
    .and_then(|(version, target)| {
        let signature = PathBuf::from(format!("{}.minisig", path.display()));
        read_signature(&signature).and_then(|signature| {
            File::open(path)
                .map_err(|error| eyre!("Cannot open bundle archive: {error}"))
                .and_then(|mut archive| verify_bundle(&mut archive, &signature, &version, target).map(|_| ()))
        })
    })
}

/// Read a detached signature with a strict size limit.
pub(crate) fn read_signature(path: &Path) -> Result<String> {
    File::open(path)
        .map_err(|error| eyre!("Cannot open detached bundle signature: {error}"))
        .and_then(|file| {
            let mut signature = String::new();
            file.take(MAX_SIGNATURE_BYTES + 1)
                .read_to_string(&mut signature)
                .map_err(|error| eyre!("Cannot read detached bundle signature: {error}"))
                .and_then(|_| {
                    (signature.len() as u64 <= MAX_SIGNATURE_BYTES)
                        .then_some(signature)
                        .ok_or_else(|| eyre!("Detached bundle signature exceeds the size limit"))
                })
        })
}

/// Extract the already verified binaries from the same open archive handle.
#[cfg(all(unix, any(target_os = "linux", test)))]
pub(crate) fn extract_verified(archive: &mut File, directory: &Path) -> Result<()> {
    archive
        .seek(SeekFrom::Start(0))
        .map_err(|error| eyre!("Cannot rewind verified bundle: {error}"))
        .and_then(|_| {
            Archive::new(GzDecoder::new(archive))
                .entries()
                .map_err(|error| eyre!("Cannot extract verified bundle: {error}"))
                .and_then(|entries| {
                    entries.into_iter().try_for_each(|entry| {
                        entry
                            .map_err(|error| eyre!("Cannot read verified bundle member: {error}"))
                            .and_then(|mut entry| match entry.path_bytes().as_ref() {
                                | b"acorn" | b"acornd" => {
                                    let name = String::from_utf8_lossy(entry.path_bytes().as_ref()).into_owned();
                                    let path = directory.join(&name);
                                    OpenOptions::new()
                                        .write(true)
                                        .create_new(true)
                                        .mode(0o700)
                                        .open(&path)
                                        .and_then(|mut file| std::io::copy(&mut entry, &mut file).and_then(|_| file.sync_all()))
                                        .map_err(|error| eyre!("Cannot extract verified binary {}: {error}", path.display()))
                                }
                                | b"manifest.json" => Ok(()),
                                | _ => Err(eyre!("Unexpected member in verified bundle")),
                            })
                    })
                })
        })
}

fn verify_with_key<R: Read + Seek>(
    archive: &mut R,
    signature: &str,
    expected_version: &str,
    expected_target: &str,
    key: &PublicKey,
) -> Result<BundleManifest> {
    Signature::decode(signature)
        .map_err(|error| eyre!("Bundle signature is invalid: {error}"))
        .and_then(|parsed| verify_signature(archive, &parsed, key))
        .and_then(|()| {
            archive
                .seek(SeekFrom::Start(0))
                .map_err(|error| eyre!("Cannot rewind verified bundle: {error}"))
        })
        .and_then(|_| inspect_archive(archive))
        .and_then(|contents| validate_contents(contents, expected_version, expected_target))
}

fn verify_signature<R: Read + Seek>(archive: &mut R, signature: &Signature, key: &PublicKey) -> Result<()> {
    archive
        .seek(SeekFrom::Start(0))
        .map_err(|error| eyre!("Cannot seek bundle: {error}"))
        .and_then(|_| {
            key.verify_stream(signature)
                .map_err(|error| eyre!("Bundle signature is unsupported: {error}"))
        })
        .and_then(|mut verifier| {
            let mut buffer = [0u8; 64 * 1024];
            let mut total = 0u64;
            let mut outcome = Ok(());
            loop {
                match archive.read(&mut buffer) {
                    | Ok(0) => break,
                    | Ok(count) => {
                        total = total.saturating_add(count as u64);
                        if total > MAX_BUNDLE_BYTES {
                            outcome = Err(eyre!("Bundle exceeds the size limit"));
                            break;
                        }
                        match buffer.get(..count) {
                            | Some(bytes) => verifier.update(bytes),
                            | None => {
                                outcome = Err(eyre!("Bundle read exceeded its buffer"));
                                break;
                            }
                        }
                    }
                    | Err(error) => {
                        outcome = Err(eyre!("Cannot read bundle: {error}"));
                        break;
                    }
                }
            }
            outcome.and_then(|()| {
                verifier
                    .finalize()
                    .map_err(|error| eyre!("Bundle signature verification failed: {error}"))
            })
        })
}

fn inspect_archive<R: Read>(reader: R) -> Result<Contents> {
    let mut archive = Archive::new(GzDecoder::new(reader));
    archive
        .entries()
        .map_err(|error| eyre!("Cannot read bundle archive: {error}"))
        .and_then(|mut entries| {
            entries.try_fold(Contents::default(), |mut contents, entry| {
                entry
                    .map_err(|error| eyre!("Cannot read bundle member: {error}"))
                    .and_then(|mut entry| {
                        let name = entry.path_bytes();
                        let regular = entry.header().entry_type().is_file();
                        if !regular {
                            Err(eyre!("Bundle contains a non-file member"))
                        } else {
                            match name.as_ref() {
                                | b"manifest.json" => read_manifest(&mut entry, &mut contents),
                                | b"acorn" | b"acornd" | b"acorn.exe" | b"acornd.exe" => {
                                    let name = String::from_utf8_lossy(name.as_ref()).into_owned();
                                    read_binary(&mut entry, name, &mut contents)
                                }
                                | _ => Err(eyre!("Bundle contains an unexpected or unsafe member")),
                            }
                        }
                    })
                    .map(|()| contents)
            })
        })
        .and_then(|contents| {
            std::io::copy(&mut archive.into_inner(), &mut std::io::sink())
                .map_err(|error| eyre!("Cannot finish reading bundle archive: {error}"))
                .map(|_| contents)
        })
}

fn read_manifest<R: Read>(entry: &mut tar::Entry<'_, R>, contents: &mut Contents) -> Result<()> {
    if contents.manifest.is_some() || entry.size() > MAX_MANIFEST_BYTES {
        Err(eyre!("Bundle manifest is duplicated or too large"))
    } else {
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| eyre!("Cannot read bundle manifest: {error}"))
            .and_then(|count| {
                (count as u64 == entry.size())
                    .then_some(bytes)
                    .ok_or_else(|| eyre!("Bundle manifest is truncated"))
            })
            .and_then(|bytes| serde_json::from_slice::<BundleManifest>(&bytes).map_err(|error| eyre!("Bundle manifest is invalid: {error}")))
            .map(|manifest| {
                contents.manifest = Some(manifest);
            })
    }
}

fn read_binary<R: Read>(entry: &mut tar::Entry<'_, R>, name: String, contents: &mut Contents) -> Result<()> {
    if contents.binaries.contains_key(&name) || entry.size() > MAX_BINARY_BYTES {
        Err(eyre!("Bundle binary is duplicated or too large: {name}"))
    } else {
        let mut digest = Context::new(&SHA256);
        let mut writer = DigestWriter(&mut digest);
        std::io::copy(entry, &mut writer)
            .map_err(|error| eyre!("Cannot read bundle binary {name}: {error}"))
            .and_then(|count| {
                (count == entry.size())
                    .then_some(count)
                    .ok_or_else(|| eyre!("Bundle binary is truncated: {name}"))
            })
            .map(|count| {
                contents.binaries.insert(name, (count, HEXLOWER.encode(digest.finish().as_ref())));
            })
    }
}

struct DigestWriter<'a>(&'a mut Context);

impl std::io::Write for DigestWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn validate_contents(contents: Contents, expected_version: &str, expected_target: &str) -> Result<BundleManifest> {
    contents.manifest.ok_or_else(|| eyre!("Bundle manifest is missing")).and_then(|manifest| {
        let names = if manifest.target == "x86_64-pc-windows-msvc" {
            ["acorn.exe", "acornd.exe"]
        } else {
            ["acorn", "acornd"]
        };
        let valid_header = manifest.format_version == FORMAT_VERSION
            && manifest.version == expected_version
            && manifest.target == expected_target
            && TARGETS.contains(&manifest.target.as_str());
        let valid_binaries = manifest.binaries.len() == names.len()
            && manifest.binaries.iter().zip(names).all(|(binary, name)| {
                binary.name == name
                    && binary.sha256.len() == 64
                    && binary.sha256.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                    && contents.binaries.get(name) == Some(&(binary.size, binary.sha256.clone()))
            });
        (valid_header && valid_binaries && contents.binaries.len() == names.len())
            .then_some(manifest)
            .ok_or_else(|| eyre!("Bundle metadata or binary checksum does not match the expected release"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use minisign::KeyPair;
    use serde_json::json;
    #[cfg(unix)]
    use std::fs;
    use std::io::{Cursor, Write};
    use tar::{Builder, Header};

    fn append(builder: &mut Builder<GzEncoder<Vec<u8>>>, name: &str, bytes: &[u8]) {
        let mut header = Header::new_gnu();
        header.as_mut_bytes().get_mut(..name.len()).unwrap().copy_from_slice(name.as_bytes());
        header.set_size(bytes.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder.append(&header, bytes).unwrap();
    }

    fn bundle(version: &str, target: &str, members: &[(&str, &[u8])]) -> Vec<u8> {
        let manifest = json!({
            "formatVersion": 1,
            "version": version,
            "target": target,
            "binaries": members.iter().map(|(name, bytes)| json!({
                "name": name,
                "sha256": HEXLOWER.encode(ring::digest::digest(&SHA256, bytes).as_ref()),
                "size": bytes.len(),
            })).collect::<Vec<_>>(),
        });
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut builder = Builder::new(encoder);
        append(&mut builder, "manifest.json", &serde_json::to_vec(&manifest).unwrap());
        for (name, bytes) in members {
            append(&mut builder, name, bytes);
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    fn signed(bytes: &[u8]) -> (PublicKey, String) {
        let keys = KeyPair::generate_unencrypted_keypair().unwrap();
        let public = PublicKey::decode(&keys.pk.to_box().unwrap().into_string()).unwrap();
        let signature = minisign::sign(Some(&keys.pk), &keys.sk, Cursor::new(bytes), Some("ACORN test"), None)
            .unwrap()
            .into_string();
        (public, signature)
    }

    fn verify(bytes: &[u8], key: &PublicKey, signature: &str, version: &str) -> Result<BundleManifest> {
        verify_with_key(&mut Cursor::new(bytes), signature, version, "x86_64-unknown-linux-musl", key)
    }

    #[test]
    fn pinned_public_key_parses() {
        assert!(PublicKey::decode(PUBLIC_KEY).is_ok());
    }

    #[test]
    fn accepts_signed_complete_bundle() {
        let bytes = bundle("0.6.3", "x86_64-unknown-linux-musl", &[("acorn", b"cli"), ("acornd", b"daemon")]);
        let (key, signature) = signed(&bytes);
        assert_eq!(verify(&bytes, &key, &signature, "0.6.3").unwrap().version, "0.6.3");
    }

    #[test]
    fn rejects_wrong_key_and_changed_archive() {
        let bytes = bundle("0.6.3", "x86_64-unknown-linux-musl", &[("acorn", b"cli"), ("acornd", b"daemon")]);
        let (_, signature) = signed(&bytes);
        let (wrong_key, _) = signed(&bytes);
        assert!(verify(&bytes, &wrong_key, &signature, "0.6.3").is_err());
        let (key, signature) = signed(&bytes);
        let mut changed = bytes;
        *changed.get_mut(12).unwrap() ^= 1;
        assert!(verify(&changed, &key, &signature, "0.6.3").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn extracts_from_the_verified_handle_after_path_replacement() {
        let bytes = bundle(
            "0.6.3",
            "x86_64-unknown-linux-musl",
            &[("acorn", b"original cli"), ("acornd", b"original daemon")],
        );
        let (key, signature) = signed(&bytes);
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("bundle.tar.gz");
        fs::write(&path, &bytes).unwrap();
        let mut handle = File::open(&path).unwrap();
        verify_with_key(&mut handle, &signature, "0.6.3", "x86_64-unknown-linux-musl", &key).unwrap();
        fs::rename(&path, temporary.path().join("verified.tar.gz")).unwrap();
        fs::write(&path, b"replacement").unwrap();
        extract_verified(&mut handle, temporary.path()).unwrap();
        assert_eq!(fs::read(temporary.path().join("acorn")).unwrap(), b"original cli");
        assert_eq!(fs::read(temporary.path().join("acornd")).unwrap(), b"original daemon");
    }

    #[test]
    fn rejects_missing_member_version_and_target() {
        let bytes = bundle("0.6.3", "x86_64-unknown-linux-musl", &[("acorn", b"cli")]);
        let (key, signature) = signed(&bytes);
        assert!(verify(&bytes, &key, &signature, "0.6.3").is_err());
        let bytes = bundle("0.6.3", "x86_64-unknown-linux-musl", &[("acorn", b"cli"), ("acornd", b"daemon")]);
        let (key, signature) = signed(&bytes);
        assert!(verify(&bytes, &key, &signature, "0.6.4").is_err());
        assert!(verify_with_key(&mut Cursor::new(&bytes), &signature, "0.6.3", "aarch64-unknown-linux-musl", &key).is_err());
    }

    #[test]
    fn rejects_signed_path_traversal_and_checksum_mismatch() {
        let bytes = bundle("0.6.3", "x86_64-unknown-linux-musl", &[("../acorn", b"cli"), ("acornd", b"daemon")]);
        let (key, signature) = signed(&bytes);
        assert!(verify(&bytes, &key, &signature, "0.6.3").is_err());
        let mut bytes = bundle("0.6.3", "x86_64-unknown-linux-musl", &[("acorn", b"cli"), ("acornd", b"daemon")]);
        let mut expanded = Vec::new();
        GzDecoder::new(Cursor::new(&bytes)).read_to_end(&mut expanded).unwrap();
        let location = expanded.windows(3).position(|window| window == b"cli").unwrap();
        *expanded.get_mut(location).unwrap() ^= 1;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&expanded).unwrap();
        bytes = encoder.finish().unwrap();
        let (key, signature) = signed(&bytes);
        assert!(verify(&bytes, &key, &signature, "0.6.3").unwrap_err().to_string().contains("checksum"));
    }
}
