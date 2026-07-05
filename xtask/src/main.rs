use std::process::Command;
use toml_edit::{value, DocumentMut};

const WORKSPACE_MANIFEST: &str = "Cargo.toml";
struct InternalDep {
    manifest: &'static str,
    name: &'static str,
}
const INTERNAL_DEPS: &[InternalDep] = &[
    InternalDep {
        manifest: "acorn-cli/Cargo.toml",
        name: "acorn-lib",
    },
    InternalDep {
        manifest: "acorn-cli/Cargo.toml",
        name: "acorn-tui",
    },
    InternalDep {
        manifest: "acorn-lib/Cargo.toml",
        name: "acorn-macros",
    },
    InternalDep {
        manifest: "acorn-py/Cargo.toml",
        name: "acorn-lib",
    },
    InternalDep {
        manifest: "acorn-tui/Cargo.toml",
        name: "acorn-lib",
    },
];
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command>");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  release-version <version>  Update workspace version and sync internal deps");
        eprintln!("  check-version              Verify all internal dep pins match workspace version");
        std::process::exit(1);
    }
    let result = match args[1].as_str() {
        | "release-version" => {
            if args.len() < 3 {
                eprintln!("Error: missing version argument");
                eprintln!("Usage: cargo xtask release-version <version>");
                std::process::exit(1);
            }
            cmd_release_version(&args[2])
        }
        | "check-version" => cmd_check_version(),
        | _ => {
            eprintln!("Error: unknown command '{}'", args[1]);
            eprintln!("Available commands: release-version, check-version");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
fn validate_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}
fn read_toml(path: &str) -> Result<DocumentMut, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;
    content.parse::<DocumentMut>().map_err(|e| format!("failed to parse {path}: {e}"))
}
fn write_toml(path: &str, doc: &DocumentMut) -> Result<(), String> {
    std::fs::write(path, doc.to_string()).map_err(|e| format!("failed to write {path}: {e}"))
}
fn get_workspace_version(doc: &DocumentMut) -> Result<String, String> {
    let v = doc["workspace"]["package"]["version"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "could not find [workspace.package].version in Cargo.toml".to_string())?;
    Ok(v.trim_matches('"').to_string())
}
fn set_workspace_version(doc: &mut DocumentMut, version: &str) {
    doc["workspace"]["package"]["version"] = value(version);
}
fn get_dep_version(doc: &DocumentMut, name: &str) -> Option<String> {
    let entry = doc.get("dependencies")?.get(name)?;
    if let Some(table) = entry.as_inline_table() {
        table.get("version").and_then(|v| v.as_str()).map(String::from)
    } else {
        entry.as_str().map(String::from)
    }
}
fn set_dep_version(doc: &mut DocumentMut, name: &str, version: &str) -> Result<(), String> {
    let deps = doc
        .get_mut("dependencies")
        .ok_or_else(|| "no [dependencies] table in manifest".to_string())?;
    let dep = deps
        .get_mut(name)
        .ok_or_else(|| format!("dependency '{name}' not found in [dependencies]"))?;
    if dep.get("version").is_some() {
        dep["version"] = value(version);
    } else if dep.as_str().is_some() {
        *dep = value(version);
    } else {
        return Err(format!("unexpected format for dependency '{name}': expected string or inline table"));
    }
    Ok(())
}
fn update_citation_cff(version: &str) -> Result<(), String> {
    let path = "CITATION.cff";
    let content = std::fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;
    let version_line = format!("version: {version}");
    if content.contains("\nversion:") || content.starts_with("version:") {
        let mut lines: Vec<String> = content
            .lines()
            .map(|l| {
                if l.starts_with("version:") {
                    version_line.clone()
                } else {
                    l.to_string()
                }
            })
            .collect();
        lines.push(String::new());
        let output = lines.join("\n");
        std::fs::write(path, &output).map_err(|e| format!("failed to write {path}: {e}"))?;
    } else {
        let needle = "title: ACORN\n";
        let replacement = format!("title: ACORN\n{version_line}\n");
        if content.contains(needle) {
            let updated = content.replacen(needle, &replacement, 1);
            std::fs::write(path, &updated).map_err(|e| format!("failed to write {path}: {e}"))?;
        } else {
            let updated = format!("{content}\n{version_line}\n");
            std::fs::write(path, &updated).map_err(|e| format!("failed to write {path}: {e}"))?;
        }
    }
    Ok(())
}
fn cmd_release_version(version_arg: &str) -> Result<(), String> {
    let version = version_arg.strip_prefix('v').unwrap_or(version_arg);
    if !validate_semver(version) {
        return Err(format!("invalid version '{version_arg}': expected format X.Y.Z (e.g., 0.1.59)"));
    }
    let current_ws_version = {
        let doc = read_toml(WORKSPACE_MANIFEST)?;
        get_workspace_version(&doc)?
    };
    if current_ws_version == version {
        println!("Already at version {version}, nothing to do.");
        return Ok(());
    }
    println!("Workspace version: {current_ws_version} → {version}");
    println!();
    let mut root_doc = read_toml(WORKSPACE_MANIFEST)?;
    set_workspace_version(&mut root_doc, version);
    write_toml(WORKSPACE_MANIFEST, &root_doc)?;
    println!("  ✓ updated [workspace.package].version in Cargo.toml");
    for dep in INTERNAL_DEPS {
        let mut doc = read_toml(dep.manifest)?;
        match get_dep_version(&doc, dep.name) {
            | Some(dep_ver) if dep_ver == version => {
                println!("  ✓ {m}:{d} already at {version}", m = dep.manifest, d = dep.name);
            }
            | Some(dep_ver) => {
                set_dep_version(&mut doc, dep.name, version)?;
                write_toml(dep.manifest, &doc)?;
                println!("  ✓ {m}:{d} {dep_ver} → {version}", m = dep.manifest, d = dep.name);
            }
            | None => {
                println!("  - {m}:{d} (dep not found, skipped)", m = dep.manifest, d = dep.name);
            }
        }
    }
    update_citation_cff(version)?;
    println!("  ✓ updated version in CITATION.cff");
    println!();
    println!("Running cargo update -w...");
    let status = Command::new("cargo")
        .args(["update", "-w"])
        .status()
        .map_err(|e| format!("failed to run cargo update: {e}"))?;
    if !status.success() {
        return Err("cargo update failed".to_string());
    }
    println!();
    println!("Done! Workspace updated to version {version}.");
    Ok(())
}
fn cmd_check_version() -> Result<(), String> {
    let root_doc = read_toml(WORKSPACE_MANIFEST)?;
    let workspace_version = get_workspace_version(&root_doc)?;
    println!("Workspace version: {workspace_version}");
    println!();
    let mut all_ok = true;
    for dep in INTERNAL_DEPS {
        let doc = read_toml(dep.manifest)?;
        match get_dep_version(&doc, dep.name) {
            | Some(ver) if ver == workspace_version => {
                println!("  ✓ {m}:{d} = {ver}", m = dep.manifest, d = dep.name);
            }
            | Some(ver) => {
                println!(
                    "  ✗ {m}:{d} = {ver} (expected {expected})",
                    m = dep.manifest,
                    d = dep.name,
                    expected = workspace_version
                );
                all_ok = false;
            }
            | None => {
                println!("  ? {m}:{d} = (not found)", m = dep.manifest, d = dep.name);
                all_ok = false;
            }
        }
    }
    println!();
    if all_ok {
        println!("All internal dependency pins match workspace version ✓");
        Ok(())
    } else {
        Err("Some internal dependency pins are out of sync with workspace version.".to_string())
    }
}
