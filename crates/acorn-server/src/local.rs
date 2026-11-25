//! Owner-only local control socket paths.
use nix::unistd::geteuid;
use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}
fn directory_metadata(path: &Path, owner_only: bool) -> io::Result<()> {
    fs::symlink_metadata(path).and_then(|metadata| {
        let mode = metadata.permissions().mode();
        let safe_mode = if owner_only { mode & 0o077 == 0 } else { mode & 0o022 == 0 };
        let current_uid = geteuid().as_raw();
        let safe_owner = metadata.uid() == current_uid || (!owner_only && metadata.uid() == 0);
        (metadata.file_type().is_dir() && safe_mode && safe_owner)
            .then_some(())
            .ok_or_else(|| invalid("Local daemon socket directory is not owner-protected"))
    })
}
/// Prepare an owner-only directory for a local daemon control socket.
pub fn prepare_private_dir(path: &Path) -> io::Result<()> {
    path.is_absolute()
        .then_some(())
        .ok_or_else(|| invalid("Local daemon socket directory must be absolute"))
        .and_then(|()| fs::create_dir_all(path))
        .and_then(|()| path.ancestors().skip(1).try_for_each(|ancestor| directory_metadata(ancestor, false)))
        .and_then(|()| directory_metadata(path, false))
        .and_then(|()| fs::set_permissions(path, fs::Permissions::from_mode(0o700)))
        .and_then(|()| verify_private_dir(path))
}
/// Require an owner-only directory and non-writable, nonsymlink ancestors.
pub fn verify_private_dir(path: &Path) -> io::Result<()> {
    path.is_absolute()
        .then_some(())
        .ok_or_else(|| invalid("Local daemon socket directory must be absolute"))
        .and_then(|()| directory_metadata(path, true))
        .and_then(|()| path.ancestors().skip(1).try_for_each(|ancestor| directory_metadata(ancestor, false)))
}
/// Locate the local control socket for one configured RPC port.
#[must_use]
pub fn socket_path(directory: &Path, port: u16) -> PathBuf {
    directory.join(format!("rpc-{port}.sock"))
}

#[cfg(test)]
mod tests {
    use super::{prepare_private_dir, verify_private_dir};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_rejects_writable_socket_directory() {
        let directory = tempfile::tempdir_in(std::env::var_os("HOME").expect("home directory")).unwrap();
        let data = directory.path().join("data");
        prepare_private_dir(&data).unwrap();
        assert!(verify_private_dir(&data).is_ok());
        fs::set_permissions(&data, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(verify_private_dir(&data).is_err());
        fs::set_permissions(&data, fs::Permissions::from_mode(0o700)).unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o777)).unwrap();
        assert!(verify_private_dir(&data).is_err());
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let link = directory.path().join("link");
        std::os::unix::fs::symlink(&data, &link).unwrap();
        assert!(verify_private_dir(&link).is_err());
    }
}
