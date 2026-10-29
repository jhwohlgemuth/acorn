#![no_std]

extern crate std;

use acorn_macros::bigo;

fn main() {
    const NOTATION: &str = bigo!("O(n log n)");
    let _ = NOTATION;
}
