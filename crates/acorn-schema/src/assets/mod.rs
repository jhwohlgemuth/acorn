//! Build-generated schema assets.

/// Regular-expression patterns compiled from Pomsky sources.
pub mod patterns {
    include!(concat!(env!("OUT_DIR"), "/patterns.rs"));
}

#[cfg(test)]
mod tests;
