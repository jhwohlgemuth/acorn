use crate::cli::arguments::OutputFormat;
use crate::cli::{arguments, Arguments, CommandOptions, Commands, CreateCommands, DownloadCommands, ImportCommands, ServeCommands, Void};
use crate::io::{is_stdout_piped, read_stdin, write_stdout};
use acorn::fail;
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
pub mod serve;
pub mod skill;
pub mod sync;
pub mod tui;

pub use db::{initialize_database, DatabaseConfig};
use download::model::Whitelist;

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
            | Some(CreateCommands::Bot(args)) => with_runtime(async {
                initialize_database(config).await?;
                let bind = args.common.bind_address_if_configured();
                let poll_interval = Some(args.common.poll_interval);
                create::bot::run(
                    &args.common.identifier,
                    &args.name,
                    &args.image,
                    &args.common.runtime,
                    &bind,
                    &poll_interval,
                    &args.common.after,
                    &args.domain,
                    args.common.event_source,
                    &args.common.public_url,
                    args.common.register_webhook,
                    &args.volume,
                    &args.target.remote,
                )
                .await
            }),
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
                    &args.runtime,
                    &args.target.remote,
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
            match initialize_database(config).await {
                | Ok(()) => match &arguments.command {
                    | Some(DownloadCommands::Model(args)) => {
                        match Whitelist::from(args.whitelist.as_slice()).resolve(&args.whitelist_file, offline).await {
                            | Ok(whitelist) => {
                                download::model::run(
                                    &args.model,
                                    &args.model_file,
                                    &args.sync,
                                    args.force,
                                    &args.filter,
                                    &args.ignore,
                                    &args.config,
                                    &args.output,
                                    &config.database_path,
                                    &args.verbose,
                                    offline,
                                    args.copy,
                                    args.symlink,
                                    args.skip_verify_checksum,
                                    &whitelist.0,
                                    args.no_fallback,
                                    config.no_local_database,
                                    &args.quantization,
                                    &args.gpu_memory,
                                    args.search_limit,
                                    args.minimum_download_count,
                                    args.interactive,
                                    args.dry_run,
                                    args.raw,
                                    &args.format,
                                )
                                .await
                            }
                            | Err(why) => Err(why),
                        }
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
                },
                | Err(why) => Err(why),
            }
        }),
        | Some(Commands::Export(arguments)) => {
            if arguments.schema {
                let format = OutputFormat::from(&arguments.format);
                let format_str = format.to_string().to_lowercase();
                arguments.standard.to_schema(&format_str);
                return Ok(());
            }
            with_runtime(async {
                let arguments::Export {
                    output,
                    path,
                    branch,
                    chrome_path,
                    commit,
                    filter,
                    ignore,
                    show_aspect_labels,
                    show_aspect_scores,
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
                    chrome_path,
                    commit,
                    filter,
                    ignore,
                    *show_aspect_labels,
                    *show_aspect_scores,
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
            })
        }
        | Some(Commands::Format(arguments)) => with_runtime(async {
            let arguments::Format {
                path,
                branch,
                commit,
                filter,
                ignore,
                dry_run,
                no_color,
                merge_request,
                verbose,
                ..
            } = arguments.as_ref();
            initialize_database(config).await?;
            format::run(path, branch, commit, filter, ignore, *dry_run, *no_color, merge_request, verbose, offline).await
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
        | Some(Commands::Skill) => skill::run(),
        | Some(Commands::Sync(arguments)) => with_runtime(async { sync::run(arguments, offline).await }),
        | Some(Commands::Import { command, .. }) => match command {
            | Some(ImportCommands::Model(arguments)) => with_runtime(async {
                let config = DatabaseConfig {
                    initial_download: false,
                    no_local_database: config.no_local_database || arguments.dry_run,
                    ..config.clone()
                };
                match initialize_database(&config).await {
                    | Ok(()) => {
                        import::model::run(
                            &arguments.model,
                            &arguments.model_file,
                            &arguments.sync,
                            arguments.force,
                            arguments.dry_run,
                            &arguments.config,
                            &config.database_path,
                            config.no_local_database,
                            offline,
                            arguments.no_fallback,
                            arguments.search_limit,
                            arguments.minimum_download_count,
                            arguments.interactive,
                            &arguments.verbose,
                        )
                        .await
                    }
                    | Err(why) => Err(why),
                }
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
        | Some(Commands::Serve { command, .. }) => match command {
            | Some(ServeCommands::Bot(args)) => with_runtime(async {
                let bind = args.common.bind_address();
                serve::bot::run(
                    &args.common.identifier,
                    &bind,
                    args.common.after.as_deref(),
                    Some(args.common.poll_interval),
                    args.detach,
                    &args.common.runtime,
                    args.common.event_source,
                    args.common.public_url.as_deref(),
                    args.common.register_webhook,
                )
                .await
            }),
            | Some(ServeCommands::Mcp) => with_runtime(async { serve::mcp::run().await }),
            | None => Err(eyre::eyre!("No subcommand provided for 'serve'")),
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
fn with_runtime<F>(fut: F) -> Void
where
    F: Future<Output = Void>,
{
    match Runtime::new() {
        | Ok(rt) => rt.block_on(fut),
        | Err(e) => Err(eyre::eyre!("Failed to create Tokio runtime: {e}")),
    }
}
