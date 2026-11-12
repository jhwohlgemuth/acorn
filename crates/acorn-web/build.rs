//! Stage canonical assets before Cargo expands the browser's asset macros.
use std::{fs, io, process::Command};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-env-changed=ACORN_MAP_DEV_URL");
    if let Ok(url) = std::env::var("ACORN_MAP_DEV_URL") {
        if url == "http://127.0.0.1:8083" {
            println!("cargo:rustc-env=ACORN_MAP_DEV_URL={url}");
        }
    }
    println!("cargo:rerun-if-changed=../../scripts/package-acorn-skill.py");
    println!("cargo:rerun-if-changed=../acorn-cli/assets/skills/acorn");
    println!("cargo:rerun-if-changed=../../Cargo.toml");
    for name in ["HEAD", "refs", "packed-refs", "index"] {
        if let Ok(output) = Command::new("git").args(["rev-parse", "--git-path", name]).output() {
            if output.status.success() {
                println!("cargo:rerun-if-changed={}", String::from_utf8_lossy(&output.stdout).trim());
            }
        }
    }
    let status = Command::new("python3").arg("../../scripts/package-acorn-skill.py").status();
    if !status.is_ok_and(|status| status.success()) {
        return Err(io::Error::other("could not prepare the bundled ACORN skill"));
    }
    let assets = [
        ("../../docs/src/_assets/acorn.svg", "assets/generated/acorn.svg"),
        ("../../docs/src/_assets/banner_dark_mode.png", "assets/generated/banner_dark_mode.png"),
        ("../../docs/src/_assets/banner_light_mode.png", "assets/generated/banner_light_mode.png"),
        ("../acorn-wasm/web/webmcp.js", "assets/generated/webmcp.js"),
    ];
    fs::create_dir_all("assets/generated/geospatial")
        .and_then(|()| fs::create_dir_all("assets/generated/skill"))
        .and_then(|()| {
            ["manifest.json", "SKILL.md", "acorn-skill.zip"].into_iter().try_for_each(|file| {
                let source = format!("../../target/acorn-skill/{file}");
                let destination = format!("assets/generated/skill/{file}");
                println!("cargo:rerun-if-changed={destination}");
                fs::read(source).and_then(|bytes| {
                    if fs::read(&destination).ok().as_ref() == Some(&bytes) {
                        Ok(())
                    } else {
                        fs::write(destination, bytes)
                    }
                })
            })
        })
        .and_then(|()| {
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
