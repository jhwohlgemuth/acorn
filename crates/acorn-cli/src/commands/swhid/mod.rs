//! SWHID calculation command.

use crate::cli::arguments;
use crate::cli::Void;
use acorn::io::swhid::calculate;
use acorn::util::Label;
use acorn_schema::pid::SWHID;
use color_eyre::eyre::eyre;
use core::str::FromStr;

/// Calculate or verify a SWHID for a local object.
pub(crate) fn run(arguments: &arguments::Swhid) -> Void {
    let kind = arguments.kind.into();
    let reference = arguments.reference.as_deref();
    calculate(&arguments.path, kind, reference).and_then(|actual| match arguments.verify.as_deref() {
        | Some(value) => SWHID::from_str(value)
            .map_err(|why| eyre!("invalid expected SWHID: {why}"))
            .and_then(|expected| match actual.core_identifier() == expected.core_identifier() {
                | true if arguments.raw => {
                    println!("{actual}");
                    Ok(())
                }
                | true => {
                    println!("{} {actual}", Label::pass());
                    Ok(())
                }
                | false => Err(eyre!("SWHID mismatch: expected {}, calculated {actual}", expected.core_identifier())),
            }),
        | None if arguments.raw => {
            println!("{actual}");
            Ok(())
        }
        | None => {
            println!("{} {actual} ({})", Label::pass(), arguments.path.display());
            Ok(())
        }
    })
}
