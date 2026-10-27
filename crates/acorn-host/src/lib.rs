//! Concrete, feature-gated native host capabilities for ACORN consumers.
#![forbid(unsafe_code)]

#[cfg(any(feature = "source", all(feature = "auth", test)))]
extern crate alloc;

#[cfg(feature = "auth")]
pub mod auth;
#[cfg(feature = "config")]
pub mod config;
#[cfg(feature = "download")]
pub mod download;
#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "source")]
pub mod source;
pub mod terminal;
#[cfg(test)]
mod tests;
