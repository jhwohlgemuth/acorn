//! Host-side checks for the browser script catalog and its JavaScript APIs.

use alloc::collections::BTreeSet;
use regex::Regex;
use std::path::Path;

#[test]
fn test_catalog_owns_every_script_and_host_api() {
    let module = include_str!("mod.rs");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let inline = Regex::new(r#"\bjs!\s*\(\s*("(?:\\.|[^"\\])*")"#).expect("valid inline regex");
    let authored = Regex::new(r#"\bjs_file!\s*\(\s*"([a-z0-9-]+\.js)""#).expect("valid file regex");
    let host_api = Regex::new(r"\bwindow\.(acorn[A-Za-z0-9_]+)\b").expect("valid host API regex");
    let host_definition = Regex::new(r"\bwindow\.(acorn[A-Za-z0-9_]+)\s*(?:\|\|=|\?\?=|=)[^=]").expect("valid host definition regex");

    assert!(
        !Regex::new(r#"\bchannel\s*\(\s*""#).expect("valid channel regex").is_match(module),
        "channels must use a checked script macro"
    );
    assert_eq!(module.matches("document::eval(").count(), 4, "script module must own all eval sites");

    let mut scripts = Vec::new();
    for capture in inline.captures_iter(module) {
        let source: String = serde_json::from_str(&capture[1]).expect("valid Rust script literal");
        assert!(source.len() <= 240, "inline JavaScript exceeds the 240-byte extraction limit: {source}");
        scripts.push(source);
    }

    let mut referenced = BTreeSet::new();
    for capture in authored.captures_iter(module) {
        let name = capture[1].to_owned();
        assert!(referenced.insert(name.clone()), "duplicate authored script: {name}");
        scripts.push(std::fs::read_to_string(root.join("assets/scripts").join(name)).expect("authored script exists"));
    }
    let available = std::fs::read_dir(root.join("assets/scripts"))
        .expect("script asset directory exists")
        .map(|entry| entry.expect("read script asset").file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".js") && name != "ror-index-worker.js")
        .collect::<BTreeSet<_>>();
    assert_eq!(referenced, available, "authored scripts and macro references differ");
    assert_eq!(scripts.len(), 100, "browser script catalog changed without inventory review");

    let used = scripts
        .iter()
        .flat_map(|script| host_api.captures_iter(script).map(|capture| capture[1].to_owned()))
        .collect::<BTreeSet<_>>();
    let sources = ["assets", "editor"]
        .into_iter()
        .flat_map(|directory| std::fs::read_dir(root.join(directory)).expect("browser source directory exists"))
        .map(|entry| entry.expect("read browser source entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "js"))
        .map(|path| std::fs::read_to_string(path).expect("read browser source"))
        .chain(core::iter::once(
            std::fs::read_to_string(root.join("index.html")).expect("read browser entry point"),
        ))
        .chain(scripts)
        .collect::<Vec<_>>();
    let defined = sources
        .iter()
        .flat_map(|source| host_definition.captures_iter(source).map(|capture| capture[1].to_owned()))
        .collect::<BTreeSet<_>>();
    for name in used {
        assert!(defined.contains(&name), "window.{name} has no defining browser source");
    }
}
