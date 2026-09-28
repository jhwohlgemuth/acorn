let source_directory = argument(0, "");
let reference = argument(1, "");
let bundle = argument(2, "needle-modelkit");
let requested_platform = argument(3, "");
let acorn = environment("ACORN_BIN", "acorn");
let oras = environment("ORAS_BIN", "oras");
let source_revision = environment("NEEDLE_SOURCE_REVISION", "b274efcb211a9eef48c9a88da4b43bd569696a39");

if source_directory == "" || reference == "" {
    fail("Usage: create-oci-artifact.rhai SOURCE_DIRECTORY REGISTRY_REFERENCE [BUNDLE_DIRECTORY] [PLATFORM]");
}
if !directory_exists(source_directory) {
    fail(`Needle 3 source directory does not exist: ${source_directory}`);
}
if file_exists(bundle) || directory_exists(bundle) {
    fail(`Refusing to overwrite bundle path: ${bundle}`);
}
if !hexadecimal(source_revision, 40) {
    fail("NEEDLE_SOURCE_REVISION must be a 40-character hexadecimal revision");
}

let platform = if requested_platform == "" { host_platform() } else { requested_platform };
let runner_name = switch platform {
    "windows-x86_64" | "windows-arm64" => "needle.exe",
    "linux-x86_64" | "linux-arm64" | "macos-arm64" => "needle",
    _ => fail(`Unsupported Needle platform: ${platform}`)
};
let runner = join_path(source_directory, runner_name);
let model = join_path(source_directory, "needle3.cact");
let license = join_path(source_directory, "LICENSE");
for required_file in [runner, model, license] {
    if !file_exists(required_file) {
        fail(`Needle 3 bundle input does not exist: ${required_file}`);
    }
}

let registry_reference = strip_prefix(reference, "oci://");
if contains_text(registry_reference, "@") {
    fail(`Publish to a tag, not an existing digest: ${reference}`);
}

run(acorn, ["export", "--tools", "--output", bundle]);
let bundle_runner = join_path(bundle, runner_name);
let bundle_model = join_path(bundle, "needle3.cact");
let bundle_license = join_path(bundle, "LICENSE");
let tools = join_path(bundle, "tools.json");
copy_file(runner, bundle_runner);
copy_file(model, bundle_model);
copy_file(license, bundle_license);

let manifest = #{
    manifestVersion: "v1.0.0",
    sourceRevision: source_revision,
    platform: platform,
    sidecar: #{ kind: "needle", protocolVersion: 2 },
    runner: #{ path: runner_name, size: file_size(bundle_runner), sha256: sha256(bundle_runner) },
    assets: [
        #{ name: "license", path: "LICENSE", size: file_size(bundle_license), sha256: sha256(bundle_license) },
        #{ name: "model", path: "needle3.cact", size: file_size(bundle_model), sha256: sha256(bundle_model) },
        #{ name: "tools", path: "tools.json", size: file_size(tools), sha256: sha256(tools) }
    ],
    tools: "tools"
};
write_text(join_path(bundle, "needle-modelkit.json"), json(manifest));

let transport = [];
let registry_config = environment("ORAS_REGISTRY_CONFIG", "");
let ca_file = environment("ORAS_CA_FILE", "");
if registry_config != "" {
    transport = array_append(transport, ["--registry-config", registry_config]);
}
if ca_file != "" {
    transport = array_append(transport, ["--ca-file", ca_file]);
}
if environment("ORAS_PLAIN_HTTP", "0") == "1" {
    transport = array_append(transport, ["--plain-http"]);
}

let push = [
    "push", "--artifact-type", "application/vnd.acorn.needle.modelkit.v2", registry_reference,
    "needle-modelkit.json:application/vnd.acorn.needle.modelkit.manifest.v2+json",
    "LICENSE:text/plain", "needle3.cact:application/vnd.acorn.needle.model.v3", "tools.json:application/json",
    `${runner_name}:application/vnd.acorn.needle.runner.v3`
];
push = array_append(push, transport);
run(oras, push);

let resolve = ["resolve", "--full-reference", registry_reference];
resolve = array_append(resolve, transport);
let resolved_reference = run(oras, resolve);
if !has_sha256_digest(resolved_reference) {
    fail(`ORAS returned an unexpected resolved reference: ${resolved_reference}`);
}

print(`Bundle: ${bundle}`);
print(`Custom location: oci://${resolved_reference}`);
print(`Run: acorn infer "Return the ACORN version" --custom-location oci://${resolved_reference}`);
