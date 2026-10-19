use crate::{
    cli::{
        arguments, Arguments, CommandOptions, Commands, CreateCommands, DownloadCommands, ExecCommands, ExecHookCommands, ExecManifestCommands,
        ImportCommands, InferCommands, IntakeCommands, ServeCommands, Void, WatchOptions,
    },
    io::{is_stdout_piped, read_stdin, write_stdout},
};
use acorn::{
    analyzer::host::{context, discovery},
    fail,
    io::ContainerTarget,
};
use acorn_core::options::CommonRuntime;
use acorn_host::terminal::Label;
use acorn_schema::discovery::RemoteOrganizationRole;
use acorn_schema::standard::Standard;
use acorn_schema::vocabulary::VocabularyFuzzyMode;
use clap_verbosity_flag::VerbosityFilter;
use color_eyre::{eyre, eyre::eyre};
use core::future::Future;
pub use db::{initialize_database, initialize_database_with_policy, DatabaseConfig};
use download::model::Whitelist;
use export::export_schema;
use jiff::SignedDuration;
use tokio::runtime::Runtime;
use tracing::info;
pub mod check;
pub mod create;
pub mod db;
pub mod diff;
pub mod doctor;
pub mod download;
pub mod exec;
pub mod export;
pub mod format;
pub mod gather;
pub mod import;
pub mod infer;
pub mod intake;
pub mod link;
pub mod mcp;
pub mod report;
pub mod serve;
pub mod skill;
pub mod swhid;
pub mod sync;
pub mod tui;
pub mod watch;
macro_rules! preflight {
    ($options:expr) => {
        match $crate::commands::preflight_inner(module_path!().rsplit("::").next().unwrap_or(module_path!()), $options) {
            | Ok(()) => {}
            | Err(e) => return Err(e),
        }
    };
}
pub(crate) use preflight;
fn patch_without_suggestions(arguments: &arguments::Check) -> bool {
    !arguments.suggest_fixes && arguments.output.is_patch()
}
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
pub(crate) fn resolve_poll_interval(args: &Arguments) -> SignedDuration {
    let command_interval = match &args.command {
        | Some(Commands::Check(arguments)) => arguments.poll_interval,
        | Some(Commands::Gather(arguments)) => arguments.poll_interval,
        | _ => None,
    };
    command_interval.or(args.poll_interval).unwrap_or_else(|| SignedDuration::from_secs(1))
}
pub fn run(args: &Arguments, offline: bool, config: &DatabaseConfig, threads: usize) -> Void {
    let poll_interval = resolve_poll_interval(args);
    let options = WatchOptions {
        mode: args.watch_mode,
        poll_interval,
        network: false,
    };
    let watch = args.watch.then_some(options);
    validate_watch_arguments(args)
        .and_then(|()| validate_oci_standard_arguments(args))
        .and_then(|()| run_command(args, offline, config, threads, watch))
}
fn run_command(args: &Arguments, offline: bool, config: &DatabaseConfig, threads: usize, watch: Option<WatchOptions>) -> Void {
    match &args.command {
        | Some(Commands::Check(arguments)) => with_runtime(async {
            let arguments::Check {
                path,
                paste,
                branch,
                commit,
                config: application_config,
                filter,
                format,
                ignore,
                skip,
                disable_website_checks,
                all,
                exit_on_first_error,
                ignore_sync_failure,
                merge_request,
                skip_verify_checksum,
                no_fail,
                raw,
                terse,
                vale_directory,
                standard,
                readability_metric,
                fuzzy,
                suggest_fixes,
                output,
                inference,
                verbose,
                ..
            } = arguments.as_ref();
            initialize_database_with_policy(config, true).await?;
            check::run(
                path,
                paste,
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
                fuzzy,
                verbose,
                offline,
                watch,
                *ignore_sync_failure,
                resolve_poll_interval(args),
                vale_directory,
                format,
                application_config,
                *suggest_fixes,
                *output,
                inference,
            )
            .await
        }),
        | Some(Commands::Create { command, .. }) => match command {
            | Some(CreateCommands::Mcp(arguments)) => create::mcp::run(arguments),
            | Some(CreateCommands::Bot(args)) => with_runtime(async {
                initialize_database(config).await?;
                let bind = args.common.bind_address_if_configured();
                let poll_interval = Some(args.common.poll_interval);
                let target = ContainerTarget::from_args(args.target.host.clone(), args.target.context.clone());
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
                    &args.common.config,
                    &args.volume,
                    target.as_ref(),
                    args.detach.detach,
                )
                .await
            }),
            | Some(CreateCommands::Runner(args)) => with_runtime(async {
                initialize_database(config).await?;
                let target = ContainerTarget::from_args(args.target.host.clone(), args.target.context.clone());
                create::runner::run(
                    &args.config,
                    args.no_config,
                    &args.description,
                    &args.name,
                    &args.server,
                    &args.repo,
                    &args.group,
                    &args.project,
                    &args.tags,
                    &args.untagged,
                    &args.runtime,
                    target.as_ref(),
                    &args.verbose,
                )
                .await
            }),
            | Some(CreateCommands::Secret { paste }) => create::secret::run(*paste),
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
        | Some(Commands::Diff(arguments)) => diff::run(arguments),
        | Some(Commands::Download(arguments)) => match arguments.unsupported_bucket_flag() {
            | Some(flag) => Err(eyre::eyre!("{flag} is only supported for bucket downloads")),
            | None => match &arguments.command {
                | Some(DownloadCommands::Chromium) => {
                    #[cfg(feature = "pdf")]
                    {
                        with_runtime(download::chromium(offline))
                    }
                    #[cfg(not(feature = "pdf"))]
                    {
                        Err(eyre::eyre!("PDF feature not enabled"))
                    }
                }
                | Some(DownloadCommands::Needle) => with_runtime(download::needle(&arguments.verbose, offline)),
                | Some(DownloadCommands::Vale) => with_runtime(download::vale(offline)),
                | _ => with_runtime(async {
                    match initialize_database(config).await {
                        | Ok(()) => match &arguments.command {
                            | Some(DownloadCommands::Model(args)) => {
                                match Whitelist::from(args.whitelist.as_slice()).resolve(&args.whitelist_file, offline).await {
                                    | Ok(whitelist) => {
                                        download::model::execute(download::model::Request {
                                            arguments: args,
                                            whitelist: &whitelist.0,
                                            database_path: &config.database_path,
                                            offline,
                                            no_local_database: config.no_local_database,
                                        })
                                        .await
                                    }
                                    | Err(why) => Err(why),
                                }
                            }
                            | Some(DownloadCommands::Needle) => download::needle(&arguments.verbose, offline).await,
                            | Some(DownloadCommands::Chromium | DownloadCommands::Vale) => Err(eyre::eyre!("Internal download dispatch error")),
                            | None => download::run(arguments, &config.database_path, threads, offline, config.no_local_database).await,
                        },
                        | Err(why) => Err(why),
                    }
                }),
            },
        },
        | Some(Commands::Export(arguments)) => match (arguments.schema, arguments.tools) {
            | (true, _) => export_schema(arguments.standard, &arguments.format).map(|output| println!("{output}")),
            | (false, true) => export::tools(arguments.output.as_deref(), arguments.dry_run),
            | (false, false) => with_runtime(async {
                let arguments::Export {
                    output,
                    path,
                    branch,
                    chrome_path,
                    commit,
                    filter,
                    full_timestamp,
                    ignore,
                    show_aspect_labels,
                    show_aspect_scores,
                    format,
                    archive_format,
                    reference,
                    from,
                    to,
                    config: export_config,
                    crosswalk_contact,
                    local_identifier,
                    combine,
                    merge_request,
                    raw,
                    skip,
                    dry_run,
                    strict,
                    target,
                    since,
                    verbose,
                    ..
                } = arguments.as_ref();
                initialize_database(config).await?;
                export::run_with_import_context(
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
                    *archive_format,
                    reference,
                    from,
                    to,
                    export_config,
                    crosswalk_contact.as_deref(),
                    local_identifier.as_deref(),
                    combine,
                    merge_request,
                    raw,
                    skip,
                    *dry_run,
                    *strict,
                    *target,
                    since.as_deref(),
                    *full_timestamp,
                    threads,
                    verbose,
                    offline,
                )
                .await
            }),
        },
        | Some(Commands::Infer(arguments)) => match &arguments.command {
            | Some(InferCommands::Run(arguments)) => with_runtime(infer::prompt::run(arguments, offline)),
            | None => with_runtime(async {
                infer::run(
                    arguments.prompt.as_deref(),
                    arguments.custom_location.as_deref(),
                    arguments.threshold,
                    arguments.max_rounds,
                    offline,
                )
                .await
            }),
        },
        | Some(Commands::Format(arguments)) => with_runtime(async {
            let arguments::Format {
                path,
                branch,
                commit,
                config: application_config,
                filter,
                ignore,
                dry_run,
                enrich,
                fuzzy,
                no_color,
                provider,
                merge_request,
                standard,
                verbose,
            } = arguments.as_ref();
            initialize_database_with_policy(config, true).await?;
            format::run(
                path,
                branch,
                commit,
                filter,
                ignore,
                application_config,
                *dry_run,
                *enrich,
                (*fuzzy).into(),
                *no_color,
                provider,
                merge_request,
                *standard,
                threads,
                verbose,
                offline,
            )
            .await
        }),
        | Some(Commands::Gather(arguments)) => with_runtime(async {
            let arguments::Gather {
                filter,
                format,
                fuzzy,
                ignore,
                input,
                max_depth,
                merge_request,
                output,
                raw,
                terse,
                osti,
                organization,
                organization_role,
                lab,
                limit,
                offset,
                all,
                enrich,
                resolve,
                citation_format,
                standard,
                text,
                verbose,
                ..
            } = arguments.as_ref();
            initialize_database(config).await?;
            let remote = osti.as_ref().map(|view| {
                let organization = lab.map(|value| value.to_string()).or_else(|| organization.clone());
                let organization_role = lab.map_or_else(
                    || organization_role.as_ref().map(Into::into).unwrap_or_default(),
                    |_| RemoteOrganizationRole::SiteOwner,
                );
                discovery::RemoteSearchRequest::init()
                    .provider(discovery::RemoteProvider::Osti)
                    .entity(view.into())
                    .queries(input.iter().chain(text.iter()).cloned().collect())
                    .maybe_organization(organization)
                    .maybe_organization_ror(lab.and_then(|value| value.ror()))
                    .organization_role(organization_role)
                    .limit(*limit)
                    .offset(*offset)
                    .all(*all)
                    .build()
            });
            let citation_format = discovery::CitationFormat::resolve(*raw, *resolve, *citation_format);
            let options = discovery::Options {
                common: CommonRuntime {
                    filter: filter.iter().cloned().collect(),
                    ignore: ignore.iter().cloned().collect(),
                    offline,
                    quiet: verbose.is_silent(),
                    ..Default::default()
                },
                output: output.clone(),
                extension: discovery::DiscoveryExtension {
                    citation_format,
                    database_path: config.database_path.clone(),
                    enrich: *enrich,
                    format: raw.then_some(context::OutputFormat::Raw).or_else(|| format.as_ref().map(Into::into)),
                    fuzzy: (*fuzzy).into(),
                    input: input.clone(),
                    max_depth: *max_depth,
                    merge_request: *merge_request,
                    no_local_database: config.no_local_database,
                    watching: watch.is_some(),
                    resolve: *resolve,
                    standard: *standard,
                    text: text.clone(),
                    terse: *terse,
                    verbosity: match verbose.filter() {
                        | VerbosityFilter::Off => None,
                        | VerbosityFilter::Error => Some(1),
                        | VerbosityFilter::Warn => Some(2),
                        | VerbosityFilter::Info => Some(3),
                        | VerbosityFilter::Debug => Some(4),
                        | VerbosityFilter::Trace => Some(5),
                    },
                    remote,
                },
            };
            gather::run(options, watch).await
        }),
        | Some(Commands::Link(arguments)) => with_runtime(async {
            let arguments::Link {
                path,
                branch,
                commit,
                config: application_config,
                ignore,
                dry_run,
                merge_request,
                verbose,
                ..
            } = arguments.as_ref();
            initialize_database_with_policy(config, true).await?;
            link::run(path, branch, commit, ignore, application_config, *dry_run, *merge_request, verbose).await
        }),
        | Some(Commands::Mcp { command, config }) => with_runtime(mcp::run(config, command, offline)),
        | Some(Commands::Skill) => skill::run(),
        | Some(Commands::Sync(arguments)) => {
            with_runtime(async { sync::run(arguments, &config.database_path, config.no_local_database, offline).await })
        }
        | Some(Commands::Import(arguments)) if arguments.archive_format.is_some() && arguments.command.is_some() => {
            Err(eyre::eyre!("--archive-format is only supported for bucket imports"))
        }
        | Some(Commands::Import(arguments)) => match &arguments.command {
            | Some(ImportCommands::Model(arguments)) => with_runtime(async {
                let config = DatabaseConfig {
                    initial_download: false,
                    no_local_database: config.no_local_database || arguments.synchronization.dry_run,
                    ..config.clone()
                };
                match initialize_database(&config).await {
                    | Ok(()) => {
                        import::model::execute(import::model::Request {
                            arguments,
                            database_path: &config.database_path,
                            no_local_database: config.no_local_database,
                            offline,
                        })
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
            | None => with_runtime(async {
                initialize_database(config).await?;
                import::run(arguments, &config.database_path, threads, offline, config.no_local_database).await
            }),
        },
        | Some(Commands::Intake { command }) => match command {
            | IntakeCommands::Run { config } => with_runtime(intake::run(config)),
            | IntakeCommands::Export {
                config,
                submission_id,
                output,
            } => with_runtime(intake::export(config, submission_id, output)),
            | IntakeCommands::Deliver { config } => with_runtime(intake::deliver(config)),
            | IntakeCommands::Stage { config, inbox } => intake::stage(config, inbox),
            | IntakeCommands::Apply { config, submission_id } => intake::apply(config, submission_id),
        },
        | Some(Commands::Report(arguments)) => with_runtime(async {
            match initialize_database(config).await {
                | Ok(()) => report::run(arguments, offline).await,
                | Err(why) => Err(why),
            }
        }),
        | Some(Commands::Serve { command, .. }) => match command {
            | Some(ServeCommands::Bot(args)) => with_runtime(async {
                let bind = args.common.bind_address();
                let target = ContainerTarget::from_args(args.target.host.clone(), args.target.context.clone());
                serve::bot::run(
                    &args.common.identifier,
                    &bind,
                    args.common.after.as_deref(),
                    Some(args.common.poll_interval),
                    args.detach.detach,
                    &args.common.runtime,
                    target.as_ref(),
                    args.common.event_source,
                    args.common.public_url.as_deref(),
                    args.common.register_webhook,
                    args.common.config.as_deref(),
                )
                .await
            }),
            | Some(ServeCommands::Grpc(arguments)) if arguments.allow_mutation && config.no_local_database => {
                Err(eyre::eyre!("--allow-mutation requires local database persistence"))
            }
            | Some(ServeCommands::Grpc(arguments)) => with_runtime(async {
                if arguments.allow_mutation {
                    initialize_database(config).await?;
                }
                let target = ContainerTarget::from_args(arguments.target.host.clone(), arguments.target.context.clone());
                serve::grpc::run(
                    arguments.bind,
                    arguments.allow_mutation,
                    offline,
                    arguments.detach.detach,
                    target.as_ref(),
                    &arguments.runtime,
                    serve::grpc::TlsPaths::new(arguments.tls_cert.as_deref(), arguments.tls_key.as_deref()),
                )
                .await
            }),
            | Some(ServeCommands::Mcp(arguments)) if arguments.allow_mutation && config.no_local_database => {
                Err(eyre::eyre!("--allow-mutation requires local database persistence"))
            }
            | Some(ServeCommands::Mcp(arguments)) => with_runtime(async {
                if arguments.allow_mutation {
                    initialize_database(config).await?;
                }
                let target = ContainerTarget::from_args(arguments.target.host.clone(), arguments.target.context.clone());
                serve::mcp::run(
                    offline,
                    arguments.allow_mutation,
                    arguments.detach.detach,
                    config.database_path.clone(),
                    target.as_ref(),
                    &arguments.runtime,
                )
                .await
            }),
            | Some(ServeCommands::Rpc(arguments)) if arguments.allow_mutation && config.no_local_database => {
                Err(eyre::eyre!("--allow-mutation requires local database persistence"))
            }
            | Some(ServeCommands::Rpc(arguments)) => with_runtime(async {
                let target = ContainerTarget::from_args(arguments.target.host.clone(), arguments.target.context.clone());
                serve::rpc::run(
                    arguments.bind,
                    arguments.allow_mutation,
                    offline,
                    arguments.detach.detach,
                    target.as_ref(),
                    &arguments.runtime,
                )
                .await
            }),
            | None => Err(eyre::eyre!("No subcommand provided for 'serve'")),
        },
        | Some(Commands::Exec { command }) => match command {
            | Some(ExecCommands::Automation(arguments)) => exec::run_automation(&arguments.script, &arguments.arguments),
            | Some(ExecCommands::Check(arguments)) => with_runtime(async { exec::run_check(&arguments.script, &arguments.input).await }),
            | Some(ExecCommands::CheckSafe(arguments)) => exec::run_check_safe(&arguments.script, arguments.format.as_ref()),
            | Some(ExecCommands::Hook { command }) => match command {
                | Some(ExecHookCommands::Check(arguments)) => {
                    exec::run_hook_check(&arguments.script, &arguments.manifest, arguments.hook, &arguments.input, offline)
                }
                | None => Err(eyre::eyre!("No subcommand provided for 'exec hook'")),
            },
            | Some(ExecCommands::Manifest { command }) => match command {
                | Some(ExecManifestCommands::Check(arguments)) => exec::run_manifest_check(&arguments.manifest),
                | None => Err(eyre::eyre!("No subcommand provided for 'exec manifest'")),
            },
            | None => Err(eyre::eyre!("No subcommand provided for 'exec'")),
        },
        | Some(Commands::Swhid(arguments)) => swhid::run(arguments),
        | Some(Commands::Tui) => {
            with_runtime(async { initialize_database(config).await }).and_then(|_| tui::run(&config.database_path, config.no_local_database, offline))
        }
        | None if args.interactive => {
            #[cfg(feature = "tui")]
            {
                with_runtime(async { initialize_database(config).await }).and_then(|_| {
                    acorn_tui::run_with_options(
                        acorn_tui::Screen::Dashboard,
                        None,
                        acorn_tui::TuiOptions {
                            database_path: config.database_path.clone(),
                            doctor_fix: false,
                            no_local_database: config.no_local_database,
                            offline,
                            vocabulary_fuzzy: VocabularyFuzzyMode::Relaxed,
                        },
                    )
                    .map(|_| ())
                })
            }
            #[cfg(not(feature = "tui"))]
            {
                Err(eyre::eyre!("TUI feature not enabled. Build with: cargo build --features tui"))
            }
        }
        | None => {
            let name = "JASON";
            fail!("FOOBAR {name} RABOOF");
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
pub(crate) fn validate_oci_standard_arguments(args: &Arguments) -> Void {
    let unsupported = match &args.command {
        | Some(Commands::Diff(arguments)) if arguments.standard == Some(Standard::Oci) => Some("diff"),
        | Some(Commands::Export(arguments))
            if !arguments.schema
                && (arguments.standard == Standard::Oci || arguments.from == Some(Standard::Oci) || arguments.to == Some(Standard::Oci)) =>
        {
            Some("export")
        }
        | Some(Commands::Format(arguments)) if arguments.standard == Standard::Oci => Some("format"),
        | Some(Commands::Gather(arguments)) if arguments.standard == Some(Standard::Oci) => Some("gather"),
        | Some(Commands::Link(arguments)) if arguments.standard == Standard::Oci => Some("link"),
        | _ => None,
    };
    unsupported.map_or(Ok(()), |command| {
        Err(eyre!(
            "The OCI standard is only supported by the check command; '{command}' cannot use it"
        ))
    })
}
pub(crate) fn validate_watch_arguments(args: &Arguments) -> Void {
    let watch_supported = matches!(args.command, Some(Commands::Check(_) | Commands::Gather(_)));
    let ignore_sync_failure = matches!(&args.command, Some(Commands::Check(arguments)) if arguments.ignore_sync_failure);
    let paste = matches!(&args.command, Some(Commands::Check(arguments)) if arguments.paste);
    let suggest_fixes = matches!(&args.command, Some(Commands::Check(arguments)) if arguments.suggest_fixes);
    let patch_output_without_suggestions = matches!(&args.command, Some(Commands::Check(arguments)) if patch_without_suggestions(arguments));
    match (
        args.watch,
        watch_supported,
        ignore_sync_failure,
        paste,
        suggest_fixes,
        patch_output_without_suggestions,
    ) {
        | (true, false, _, _, _, _) => Err(eyre!("--watch is only supported by the check and gather commands")),
        | (false, _, true, _, _, _) => Err(eyre!("--ignore-sync-failure requires --watch")),
        | (true, _, _, true, _, _) => Err(eyre!("Clipboard text cannot be used with --watch")),
        | (true, _, _, _, true, _) => Err(eyre!("--suggest-fixes cannot be used with --watch")),
        | (_, _, _, _, _, true) => Err(eyre!("--output diff and --output json-patch require --suggest-fixes")),
        | _ => Ok(()),
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
