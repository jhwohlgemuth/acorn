use crate::cli::{arguments, Void};
use acorn::io::{
    adapter::KitfileExt,
    mcp::{
        bundle::{BundleOptions, BundlePlan, BundleReport},
        ToolRegistry,
    },
    InputOutput, Platform,
};
use acorn_host::terminal::Label;
use acorn_schema::modelkit::kitfile::Kitfile;
use color_eyre::eyre::eyre;
use owo_colors::OwoColorize;
use std::env::current_exe;
use tracing::info;

/// Create a deterministic ACORN MCP Bundle.
pub fn run(arguments: &arguments::CreateMcp) -> Void {
    let external_binary = arguments.binary.is_some();
    arguments
        .binary
        .clone()
        .map_or_else(|| current_exe().map_err(|why| eyre!("Failed to resolve running executable — {why}")), Ok)
        .and_then(|binary| Platform::resolve(arguments.platform, external_binary).map(|platform| (binary, platform)))
        .and_then(|(binary, platform)| ToolRegistry::acorn().map(|registry| (binary, platform, registry)))
        .and_then(|(binary, platform, registry)| {
            BundlePlan::new(
                &BundleOptions {
                    output: arguments.output.clone(),
                    binary,
                    platform,
                    overwrite: arguments.clobber,
                },
                &registry,
            )
        })
        .and_then(|plan| match arguments.dry_run {
            | true => serde_json::to_string_pretty(&plan.manifest).map_err(Into::into).map(|manifest| {
                println!("{manifest}");
                println!("manifest.json\n{}\noutput: {}", plan.manifest.server.entry_point, plan.output.display());
            }),
            | false => BundleReport::try_from(plan).and_then(|report| {
                arguments
                    .kitfile
                    .as_ref()
                    .map_or(Ok(()), |kitfile| {
                        Kitfile::read_yaml(kitfile.to_path_buf())
                            .and_then(|kitfile_content| kitfile_content.update(kitfile, &report.output))
                            .map_err(|why| eyre!("Bundle {} was created, but Kitfile update failed — {why}", report.output.display()))
                    })
                    .map(|()| {
                        info!(
                            "=> {} Created {} ({:?}, {} bytes, SHA-256 {})",
                            Label::pass(),
                            report.output.display().to_string().cyan(),
                            report.manifest.compatibility.platforms.first().unwrap_or(&Platform::Linux),
                            report.size,
                            report.sha256
                        );
                    })
            }),
        })
}
