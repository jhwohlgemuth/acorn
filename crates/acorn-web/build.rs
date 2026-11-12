//! Stage canonical assets before Cargo expands the browser's asset macros.
use std::{fs, io};

fn main() -> io::Result<()> {
    let assets = [
        ("../../docs/src/_assets/acorn.svg", "assets/generated/acorn.svg"),
        ("../../docs/src/_assets/banner_dark_mode.png", "assets/generated/banner_dark_mode.png"),
        ("../../docs/src/_assets/banner_light_mode.png", "assets/generated/banner_light_mode.png"),
        ("../acorn-wasm/web/webmcp.js", "assets/generated/webmcp.js"),
    ];
    fs::create_dir_all("assets/generated").and_then(|()| {
        assets.into_iter().try_for_each(|(source, destination)| {
            println!("cargo:rerun-if-changed={source}");
            println!("cargo:rerun-if-changed={destination}");
            fs::read(source).and_then(|content| {
                if fs::read(destination).is_ok_and(|existing| existing == content) {
                    Ok(())
                } else {
                    fs::write(destination, content)
                }
            })
        })
    })
}
