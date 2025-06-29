use crate::cli::Arguments;
use clap::{CommandFactory, Parser};

#[test]
fn test_cli() {
    Arguments::command().debug_assert();
    assert!(Arguments::try_parse_from(["acorn", "-V"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to/file.json"]).is_ok());
}
