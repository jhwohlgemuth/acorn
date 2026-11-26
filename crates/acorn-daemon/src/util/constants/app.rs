//! Identity and launchd registration constants for acornd.

/// Persistent daemon application name.
pub const APPLICATION: &str = "acornd";
/// System launchd label for the persistent daemon.
pub const LABEL: &str = "org.ornl.acornd";
/// Marker identifying a property list managed by ACORN.
pub(crate) const MANAGED_MARKER: &str = "<!-- Managed by ACORN -->";
/// Organization segment of the daemon identity.
pub const ORGANIZATION: &str = "ornl";
/// System launchd location reserved for the persistent daemon.
pub const PROPERTY_LIST_PATH: &str = "/Library/LaunchDaemons/org.ornl.acornd.plist";
