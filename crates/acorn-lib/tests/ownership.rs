//! Compile-time tests for canonical portable type ownership.

use acorn_core::{Location, Release, Repository, Scheme};
use acorn_schema::research_activity::ResearchActivity;

#[test]
fn test_canonical_crates_export_their_owned_types() {
    let _ = Location::from("https://example.com");
    let _ = core::mem::size_of::<Release>();
    let _ = core::mem::size_of::<Repository>();
    let _ = Scheme::HTTPS;
    let _ = ResearchActivity::default();
}

#[test]
fn test_host_library_does_not_reexport_removed_facades() {
    let source = include_str!("../src/lib.rs");
    assert!(!source.contains("pub mod schema"));
    assert!(!source.contains("pub use acorn_core::{Location"));
    assert!(!source.contains("pub use acorn_schema::"));
}
