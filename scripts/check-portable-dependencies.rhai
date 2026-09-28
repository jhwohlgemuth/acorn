let forbidden = [
    "ammonia", "anydoc", "arboard", "ariadne", "async-trait", "axum", "bat", "color-eyre", "comfy-table", "console", "directories", "dotenvy",
    "flate2", "futures", "glob", "http", "httpdate", "indicatif", "infer", "is_executable", "lychee-lib", "nanoid", "owo-colors", "pdf-inspector",
    "percy-dom", "polars", "rand", "rayon", "reqwest", "ring", "rsa", "rusqlite", "rust-embed", "rust-ini", "sevenz-rust2", "shell-words",
    "shuck-linter", "shuck-parser", "similar", "tar", "tera", "tokio", "tower", "which", "zip"
];
let forbidden_analysis = ["axum", "duckdb", "resvg", "rmcp", "shuck-linter", "shuck-parser", "sysinfo"];

fn assert_clean(label, tree, blocked) {
    let matches = tree_matches(tree, blocked);
    if array_length(matches) > 0 {
        fail(`Host-only dependencies leaked into the ${label} graph: ${json(matches)}`);
    }
}

let analysis_tree = run("cargo", ["tree", "--locked", "-p", "acorn-lib", "--no-default-features", "--features", "analysis", "--edges", "normal", "--prefix", "none"]);
assert_clean("analysis", analysis_tree, forbidden_analysis);

let native_tree = run("cargo", ["tree", "--locked", "-p", "acorn-lib", "--no-default-features", "--features", "analyzer", "--edges", "normal", "--prefix", "none"]);
assert_clean("native analyzer", native_tree, forbidden);

let wasm_tree = run("cargo", ["tree", "--locked", "-p", "acorn-lib", "--no-default-features", "--features", "analyzer", "--edges", "normal", "--prefix", "none", "--target", "wasm32-unknown-unknown"]);
assert_clean("WASM analyzer", wasm_tree, forbidden);

let wasm_package_tree = run("cargo", ["tree", "--locked", "-p", "acorn-wasm", "--edges", "normal", "--prefix", "none", "--target", "wasm32-unknown-unknown"]);
assert_clean("acorn-wasm", wasm_package_tree, forbidden);
