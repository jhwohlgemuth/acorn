//! Noninteractive signing with an encrypted, protected release key.

use minisign::{PublicKey, SecretKey};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Sign one release archive with a protected key and verify against the pinned public key.
pub(crate) fn run(args: &[String]) -> Result<(), String> {
    match args {
        | [archive] => {
            let secret_path = env::var_os("ACORND_SIGNING_KEY").ok_or_else(|| "ACORND_SIGNING_KEY file variable is required".to_string());
            let password = env::var("ACORND_SIGNING_PASSPHRASE").map_err(|_| "ACORND_SIGNING_PASSPHRASE protected variable is required".to_string());
            env::remove_var("ACORND_SIGNING_PASSPHRASE");
            secret_path.and_then(|secret_path| {
                password.and_then(|password| {
                    let public_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                        .parent()
                        .map(|root| root.join("crates/acorn-cli/assets/acornd-release.pub"))
                        .ok_or_else(|| "Cannot resolve pinned release key from the workspace".to_string());
                    public_path.and_then(|public_path| {
                        sign_file(Path::new(archive), Path::new(&secret_path), password, &public_path).map(|signature| {
                            println!("{}", signature.display());
                        })
                    })
                })
            })
        }
        | _ => Err("Usage: cargo xtask sign-acornd-bundle ARCHIVE".to_string()),
    }
}

fn sign_file(archive: &Path, secret_path: &Path, password: String, public_path: &Path) -> Result<PathBuf, String> {
    PublicKey::from_file(public_path)
        .map_err(|error| format!("Cannot read pinned release public key: {error}"))
        .and_then(|public| {
            SecretKey::from_file(secret_path, Some(password))
                .map_err(|error| format!("Cannot unlock protected release signing key: {error}"))
                .map(|secret| (public, secret))
        })
        .and_then(|(public, secret)| {
            File::open(archive)
                .map_err(|error| format!("Cannot read bundle for signing: {error}"))
                .and_then(|file| {
                    minisign::sign(Some(&public), &secret, file, Some("ACORN acornd release bundle"), None)
                        .map_err(|error| format!("Cannot sign daemon bundle: {error}"))
                        .map(|signature| (public, signature))
                })
        })
        .and_then(|(public, signature)| {
            File::open(archive)
                .map_err(|error| format!("Cannot re-read signed bundle: {error}"))
                .and_then(|file| {
                    minisign::verify(&public, &signature, file, true, false, false)
                        .map_err(|error| format!("Signed bundle did not verify under the pinned key: {error}"))
                })
                .map(|()| signature)
        })
        .and_then(|signature| {
            let mut name = archive.as_os_str().to_os_string();
            name.push(".minisig");
            let destination = PathBuf::from(name);
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .and_then(|mut file| file.write_all(signature.into_string().as_bytes()).and_then(|()| file.sync_all()))
                .map_err(|error| format!("Cannot write signed bundle signature: {error}"))
                .map(|()| destination)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use minisign::KeyPair;
    use std::fs;

    #[test]
    fn test_signs_with_matching_protected_key() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("bundle.tar.gz");
        let secret_path = dir.path().join("secret.key");
        let public_path = dir.path().join("public.pub");
        fs::write(&archive, b"test bundle").unwrap();
        let keys = KeyPair::generate_encrypted_keypair(Some("test password".to_string())).unwrap();
        fs::write(&secret_path, keys.sk.to_box(None).unwrap().to_string()).unwrap();
        fs::write(&public_path, keys.pk.to_box().unwrap().to_string()).unwrap();
        let signature = sign_file(&archive, &secret_path, "test password".to_string(), &public_path).unwrap();
        assert!(signature.exists());
        let wrong_keys = KeyPair::generate_unencrypted_keypair().unwrap();
        fs::write(&public_path, wrong_keys.pk.to_box().unwrap().to_string()).unwrap();
        assert!(sign_file(&archive, &secret_path, "test password".to_string(), &public_path).is_err());
    }
}
