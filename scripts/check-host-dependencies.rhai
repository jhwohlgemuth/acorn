let forbidden_host = ["acorn-lib", "acorn-schema", "acorn-tui", "acorn-wasm"];
let forbidden_command = ["acorn-core", "acorn-host", "acorn-lib", "acorn-schema", "acorn-tui", "acorn-wasm"];

fn check_tree(crate_name, feature, forbidden) {
    let tree = run("cargo", ["tree", "--locked", "-p", crate_name, "--no-default-features", "--features", feature, "--edges", "normal", "--prefix", "none"]);
    let matches = tree_matches(tree, forbidden);
    if array_length(matches) > 0 {
        fail(`Unrelated ACORN packages leaked into ${crate_name} [${feature}]: ${json(matches)}`);
    }
}

check_tree("acorn-cmd", "shell-lint", forbidden_command);
for feature in ["config", "download", "fs", "http", "source", "terminal"] {
    check_tree("acorn-host", feature, forbidden_host);
}
