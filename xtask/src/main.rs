mod benchmark_history;

use serde::Deserialize;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
};
use toml_edit::{value, DocumentMut};

const WORKSPACE_MANIFEST: &str = "Cargo.toml";

#[derive(Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    workspace_members: BTreeSet<String>,
}

#[derive(Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    publish: Option<Vec<String>>,
}

struct InternalDep {
    manifest: PathBuf,
    name: String,
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command>");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  benchmark-history <command>  Build or inspect persistent benchmark history");
        eprintln!("  check-release-crates <crate>...  Verify the crates.io release list");
        eprintln!("  check-version              Verify all internal dep pins match workspace version");
        eprintln!("  release-version <version>  Update workspace version and sync internal deps");
        std::process::exit(1);
    }
    let result = match args[1].as_str() {
        | "benchmark-history" => benchmark_history::run(&args[2..]),
        | "check-release-crates" => cmd_check_release_crates(&args[2..]),
        | "check-version" => cmd_check_version(),
        | "release-version" => {
            if args.len() < 3 {
                eprintln!("Error: missing version argument");
                eprintln!("Usage: cargo xtask release-version <version>");
                std::process::exit(1);
            }
            cmd_release_version(&args[2])
        }
        | _ => {
            eprintln!("Error: unknown command '{}'", args[1]);
            eprintln!("Available commands: benchmark-history, check-release-crates, check-version, release-version");
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
fn read_toml(path: impl AsRef<Path>) -> Result<DocumentMut, String> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    content
        .parse::<DocumentMut>()
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))
}
fn write_toml(path: impl AsRef<Path>, doc: &DocumentMut) -> Result<(), String> {
    let path = path.as_ref();
    std::fs::write(path, doc.to_string()).map_err(|e| format!("failed to write {}: {e}", path.display()))
}
fn get_workspace_version(doc: &DocumentMut) -> Result<String, String> {
    let v = doc["workspace"]["package"]["version"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "could not find [workspace.package].version in Cargo.toml".to_string())?;
    Ok(v.trim_matches('"').to_string())
}
fn get_crates_io_workspace_packages() -> Result<BTreeSet<String>, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| format!("failed to run cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(format!("cargo metadata failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    let metadata = serde_json::from_slice::<CargoMetadata>(&output.stdout).map_err(|e| format!("failed to parse cargo metadata: {e}"))?;
    Ok(metadata
        .packages
        .into_iter()
        .filter(|package| metadata.workspace_members.contains(&package.id))
        .filter(|package| {
            package
                .publish
                .as_ref()
                .is_none_or(|registries| registries.iter().any(|registry| registry == "crates-io"))
        })
        .map(|package| package.name)
        .collect())
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
fn get_internal_deps(root_doc: &DocumentMut, workspace_root: &Path) -> Result<Vec<InternalDep>, String> {
    let manifests = root_doc["workspace"]["members"]
        .as_array()
        .ok_or_else(|| "could not find [workspace].members in Cargo.toml".to_string())?
        .iter()
        .filter_map(|member| member.as_str())
        .map(|member| workspace_root.join(member).join("Cargo.toml"))
        .collect::<Vec<_>>();
    let package_names = manifests
        .iter()
        .map(|manifest| {
            read_toml(manifest).and_then(|doc| {
                doc["package"]["name"]
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| format!("could not find [package].name in {}", manifest.display()))
            })
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    manifests.into_iter().try_fold(Vec::new(), |mut internal_deps, manifest| {
        let doc = read_toml(&manifest)?;
        if let Some(dependencies) = doc.get("dependencies").and_then(|item| item.as_table()) {
            internal_deps.extend(dependencies.iter().filter_map(|(name, dependency)| {
                let inline = dependency.as_inline_table()?;
                let package_name = inline.get("package").and_then(|value| value.as_str()).unwrap_or(name);
                (inline.contains_key("path") && inline.contains_key("version") && package_names.contains(package_name)).then(|| InternalDep {
                    manifest: manifest.clone(),
                    name: name.to_string(),
                })
            }));
        }
        Ok(internal_deps)
    })
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
    let mut root_doc = read_toml(WORKSPACE_MANIFEST)?;
    let current_ws_version = get_workspace_version(&root_doc)?;
    let internal_deps = get_internal_deps(&root_doc, Path::new(""))?;
    if current_ws_version == version {
        println!("Workspace version: {version} (already updated)");
    } else {
        println!("Workspace version: {current_ws_version} → {version}");
    }
    println!();
    set_workspace_version(&mut root_doc, version);
    write_toml(WORKSPACE_MANIFEST, &root_doc)?;
    println!("  ✓ updated [workspace.package].version in Cargo.toml");
    for dep in internal_deps {
        let mut doc = read_toml(&dep.manifest)?;
        match get_dep_version(&doc, &dep.name) {
            | Some(dep_ver) if dep_ver == version => {
                println!("  ✓ {m}:{d} already at {version}", m = dep.manifest.display(), d = dep.name);
            }
            | Some(dep_ver) => {
                set_dep_version(&mut doc, &dep.name, version)?;
                write_toml(&dep.manifest, &doc)?;
                println!("  ✓ {m}:{d} {dep_ver} → {version}", m = dep.manifest.display(), d = dep.name);
            }
            | None => {
                println!("  - {m}:{d} (dep not found, skipped)", m = dep.manifest.display(), d = dep.name);
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
    let internal_deps = get_internal_deps(&root_doc, Path::new(""))?;
    println!("Workspace version: {workspace_version}");
    println!();
    let mut all_ok = true;
    for dep in internal_deps {
        let doc = read_toml(&dep.manifest)?;
        match get_dep_version(&doc, &dep.name) {
            | Some(ver) if ver == workspace_version => {
                println!("  ✓ {m}:{d} = {ver}", m = dep.manifest.display(), d = dep.name);
            }
            | Some(ver) => {
                println!(
                    "  ✗ {m}:{d} = {ver} (expected {expected})",
                    m = dep.manifest.display(),
                    d = dep.name,
                    expected = workspace_version
                );
                all_ok = false;
            }
            | None => {
                println!("  ? {m}:{d} = (not found)", m = dep.manifest.display(), d = dep.name);
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
fn cmd_check_release_crates(release_crates: &[String]) -> Result<(), String> {
    let expected = get_crates_io_workspace_packages()?;
    validate_release_crates(release_crates, &expected)?;
    println!("Release list includes all {} crates.io workspace crates ✓", expected.len());
    Ok(())
}
fn validate_release_crates(release_crates: &[String], expected: &BTreeSet<String>) -> Result<(), String> {
    let configured = release_crates.iter().cloned().collect::<BTreeSet<_>>();
    let duplicates = release_crates
        .iter()
        .fold((BTreeSet::new(), BTreeSet::new()), |(mut seen, mut duplicates), name| {
            if !seen.insert(name) {
                duplicates.insert(name);
            }
            (seen, duplicates)
        })
        .1;
    let missing = expected.difference(&configured).cloned().collect::<Vec<_>>();
    let unexpected = configured.difference(expected).cloned().collect::<Vec<_>>();
    let problems = [
        (!duplicates.is_empty()).then(|| format!("duplicate: {}", duplicates.into_iter().cloned().collect::<Vec<_>>().join(", "))),
        (!missing.is_empty()).then(|| format!("missing: {}", missing.join(", "))),
        (!unexpected.is_empty()).then(|| format!("unexpected: {}", unexpected.join(", "))),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "release crate list does not match crates.io workspace crates ({})",
            problems.join("; ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_deps_include_acorn_cmd() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask should be inside the workspace");
        let root_doc = read_toml(workspace_root.join(WORKSPACE_MANIFEST)).expect("workspace manifest should parse");
        let internal_deps = get_internal_deps(&root_doc, workspace_root).expect("internal dependencies should resolve");
        assert!(internal_deps
            .iter()
            .any(|dep| { dep.manifest.ends_with("crates/acorn-cli/Cargo.toml") && dep.name == "acorn-cmd" }));
    }

    #[test]
    fn test_release_crates_accepts_complete_list() {
        let expected = BTreeSet::from(["acorn-core".to_string(), "acorn-host".to_string()]);
        let release_crates = ["acorn-core".to_string(), "acorn-host".to_string()];
        assert!(validate_release_crates(&release_crates, &expected).is_ok());
    }

    #[test]
    fn test_release_crates_reports_list_mismatches() {
        let expected = BTreeSet::from(["acorn-core".to_string(), "acorn-host".to_string()]);
        let release_crates = ["acorn-core".to_string(), "acorn-core".to_string(), "not-acorn".to_string()];
        let error = validate_release_crates(&release_crates, &expected).expect_err("mismatched release list should fail");
        assert_eq!(
            error,
            "release crate list does not match crates.io workspace crates (duplicate: acorn-core; missing: acorn-host; unexpected: not-acorn)"
        );
    }
}
