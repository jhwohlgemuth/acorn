use acorn_core::prelude::alloc::{format, String};

pub mod assets;
pub mod constants;
pub mod macros;
pub use self::assets::Constant;

/// Return a deterministic alphabetic hash suitable for kebab-case suffixes.
pub(crate) fn stable_alphabetic_hash(value: impl AsRef<[u8]>) -> String {
    format!(
        "{:016x}",
        value
            .as_ref()
            .iter()
            .fold(0xcbf29ce484222325_u64, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3))
    )
    .chars()
    .map(|character| match character {
        | '0' => 'q',
        | '1' => 'r',
        | '2' => 's',
        | '3' => 't',
        | '4' => 'u',
        | '5' => 'v',
        | '6' => 'w',
        | '7' => 'x',
        | '8' => 'y',
        | '9' => 'z',
        | value => value,
    })
    .collect()
}
