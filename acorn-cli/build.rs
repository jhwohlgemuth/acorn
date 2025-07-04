#![allow(missing_docs)]
// Documentation: https://doc.rust-lang.org/cargo/reference/build-scripts.html
// Utility crate: https://github.com/richard-uk1/depgraph
fn main() {
    println!("cargo:rerun-if-changed=assets/**/*");
}
