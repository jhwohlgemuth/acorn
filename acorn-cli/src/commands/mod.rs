use crate::cli::{arguments, Arguments, CommandOptions, Commands, CreateCommands, DownloadCommands, ImportCommands, SchemaCommands, Void};
use crate::io::{is_stdout_piped, read_stdin, write_stdout};
use acorn::fail;
use acorn::schema::pid::raid::Metadata;
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::Label;
use color_eyre::eyre;
use color_eyre::eyre::eyre;
use core::future::Future;
use tokio::runtime::Runtime;
use tracing::info;

pub mod check;
pub mod create;
pub mod db;
pub mod doctor;
pub mod download;
pub mod export;
pub mod format;
pub mod gather;
pub mod import;
pub mod link;
pub mod tui;

pub use db::{initialize_database, DatabaseConfig};

macro_rules! preflight {
    ($options:expr) => {
        match $crate::commands::preflight_inner(module_path!().rsplit("::").next().unwrap_or(module_path!()), $options) {
            | Ok(()) => {}
            | Err(e) => return Err(e),
        }
    };
}
pub(crate) use preflight;

fn preflight_inner(command: &str, options: &CommandOptions) -> eyre::Result<()> {
    let CommandOptions {
        offline,
        quiet,
        supported,
        path,
        reference,
        ..
    } = options;
    let check_path = || -> eyre::Result<()> {
        path.as_ref().filter(|p| !p.exists()).map_or(Ok(()), |value| {
            eprintln!("=> {} Path not found: {}", Label::fail(), value.display());
            Err(eyre!("Path not found: {}", value.display()))
        })
    };
    let check_reference = || -> eyre::Result<()> {
        reference.as_ref().filter(|p| !p.exists()).map_or(Ok(()), |value| {
            eprintln!("=> {} Reference not found: {}", Label::fail(), value.display());
            Err(eyre!("Reference not found: {}", value.display()))
        })
    };
    let check_offline = || -> eyre::Result<()> {
        if *offline {
            if !*quiet {
                println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
            }
            if !*supported {
                let err = eyre!("Offline mode is not implemented for '{command}' command");
                eprintln!("=> {} {err}", Label::fail());
                Err(err)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    };
    check_path().and_then(|_| check_reference()).and_then(|_| check_offline())
}
pub fn run(args: &Arguments, offline: bool, config: &DatabaseConfig, threads: usize) -> Void {
    match &args.command {
        | Some(Commands::Check(arguments)) => with_runtime(async {
            let arguments::Check {
                path,
                branch,
                commit,
                filter,
                ignore,
                skip,
                disable_website_checks,
                all,
                exit_on_first_error,
                merge_request,
                skip_verify_checksum,
                no_fail,
                raw,
                terse,
                standard,
                readability_metric,
                verbose,
                ..
            } = arguments.as_ref();
            initialize_database(config).await?;
            check::run(
                path,
                branch,
                commit,
                filter,
                ignore,
                skip,
                disable_website_checks,
                all,
                exit_on_first_error,
                merge_request,
                skip_verify_checksum,
                no_fail,
                raw,
                terse,
                standard,
                readability_metric,
                verbose,
                offline,
            )
            .await
        }),
        | Some(Commands::Create { command, .. }) => match command {
            | Some(CreateCommands::Mcp) => Err(eyre::eyre!("MCP server creation is not yet implemented")),
            | Some(CreateCommands::Runner(args)) => with_runtime(async {
                initialize_database(config).await?;
                create::runner::run(
                    &args.config,
                    &args.description,
                    &args.name,
                    &args.server,
                    &args.repo,
                    &args.group,
                    &args.project,
                    &args.tags,
                    &args.untagged,
                    &args.executor,
                    &args.verbose,
                )
                .await
            }),
            | None => Err(eyre::eyre!("No subcommand provided for 'create'")),
        },
        | Some(Commands::Doctor(arguments)) => {
            let arguments::Doctor {
                fix,
                interactive,
                report,
                check,
                ..
            } = arguments.as_ref();
            doctor::run(fix, interactive, report, check, offline)
        }
        | Some(Commands::Download(arguments)) => with_runtime(async {
            initialize_database(config).await?;
            match &arguments.command {
                | Some(DownloadCommands::Model(args)) => {
                    download::model::run(
                        &args.model,
                        &args.filter,
                        &args.ignore,
                        &args.output,
                        &config.database_path,
                        &args.verbose,
                    )
                    .await
                }
                | None => {
                    download::run(
                        &arguments.config,
                        &arguments.url,
                        &arguments.filter,
                        &arguments.ignore,
                        &arguments.output,
                        &config.database_path,
                        threads,
                        &arguments.verbose,
                        offline,
                    )
                    .await
                }
            }
        }),
        | Some(Commands::Export(arguments)) => with_runtime(async {
            let arguments::Export {
                output,
                path,
                branch,
                commit,
                filter,
                ignore,
                format,
                reference,
                from,
                to,
                combine,
                merge_request,
                raw,
                skip,
                dry_run,
                strict,
                verbose,
                ..
            } = arguments.as_ref();
            initialize_database(config).await?;
            export::run(
                output,
                path,
                branch,
                commit,
                filter,
                ignore,
                format,
                reference,
                from,
                to,
                combine,
                merge_request,
                raw,
                skip,
                *dry_run,
                *strict,
                threads,
                verbose,
                offline,
            )
            .await
        }),
        | Some(Commands::Format(arguments)) => with_runtime(async {
            let arguments::Format {
                path,
                branch,
                commit,
                filter,
                ignore,
                dry_run,
                merge_request,
                verbose,
                ..
            } = arguments.as_ref();
            initialize_database(config).await?;
            format::run(path, branch, commit, filter, ignore, *dry_run, merge_request, verbose, offline).await
        }),
        | Some(Commands::Gather(arguments)) => with_runtime(async {
            let arguments::Gather {
                path,
                filter,
                ignore,
                merge_request,
                verbose,
                ..
            } = arguments.as_ref();
            initialize_database(config).await?;
            gather::run(path, filter, ignore, merge_request, &config.database_path, verbose, offline).await
        }),
        | Some(Commands::Link(arguments)) => with_runtime(async {
            let arguments::Link {
                path,
                branch,
                commit,
                ignore,
                dry_run,
                merge_request,
                verbose,
                ..
            } = arguments.as_ref();
            initialize_database(config).await?;
            link::run(path, branch, commit, ignore, *dry_run, *merge_request, verbose).await
        }),
        | Some(Commands::Import { command, .. }) => match command {
            | Some(ImportCommands::Model) => with_runtime(async {
                initialize_database(config).await?;
                import::model::run(&config.database_path).await
            }),
            | Some(ImportCommands::Spec(arguments)) => with_runtime(async {
                let arguments::Spec {
                    source,
                    name,
                    domain,
                    root,
                    auth_token,
                    output,
                    dry_run,
                    verbose,
                    ..
                } = arguments.as_ref();
                import::spec::run(source, name, domain, root, auth_token, output, *dry_run, verbose, offline).await
            }),
            | None => Err(eyre::eyre!("No subcommand provided for 'import'")),
        },
        | Some(Commands::Schema { command, .. }) => match command {
            | Some(SchemaCommands::Rad) | None => print_rad_schema(),
            | Some(SchemaCommands::Raid) => print_raid_schema(),
        },
        | Some(Commands::Tui) => tui::run(),
        | None if args.interactive => {
            #[cfg(feature = "tui")]
            return acorn_tui::run_tui(acorn_tui::Screen::Dashboard);
            #[cfg(not(feature = "tui"))]
            {
                return Err(eyre::eyre!("TUI feature not enabled. Build with: cargo build --features tui"));
            }
        }
        | None => {
            let name = "JASON";
            fail!("FOOBAR {} RABOOF", name);
            if let Some(value) = read_stdin() {
                println!("stdin: {value}");
                info!(value, "=> {} Input from stdin", Label::using());
            }
            if is_stdout_piped() {
                println!("stdout is piped");
                write_stdout("stdout is piped");
            }
            Ok(())
        }
    }
}
fn print_rad_schema() -> Void {
    ResearchActivity::to_schema();
    Ok(())
}
fn print_raid_schema() -> Void {
    Metadata::to_schema();
    Ok(())
}
fn with_runtime<F>(fut: F) -> Void
where
    F: Future<Output = Void>,
{
    match Runtime::new() {
        | Ok(rt) => rt.block_on(fut),
        | Err(e) => Err(eyre::eyre!("Failed to create Tokio runtime: {e}")),
    }
}
