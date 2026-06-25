use crate::cli::{Arguments, CommandOptions, Commands, CreateCommands, SchemaCommands, Void};
use crate::io::{is_stdout_piped, read_stdin, write_stdout};
use acorn::fail;
use acorn::prelude::{exit, PathBuf};
use acorn::schema::pid::raid::Metadata;
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::Label;
#[cfg(not(feature = "tui"))]
use color_eyre::eyre;
use tracing::info;

pub mod check;
pub mod create;
pub mod doctor;
pub mod download;
pub mod export;
pub mod format;
pub mod gather;
pub mod link;
pub mod tui;

macro_rules! preflight {
    ($options:expr) => {
        $crate::commands::preflight_inner(module_path!().rsplit("::").next().unwrap_or(module_path!()), $options)
    };
}
pub(crate) use preflight;

fn preflight_inner(command: &str, options: &CommandOptions) {
    let CommandOptions {
        offline,
        quiet,
        supported,
        path,
        reference,
        ..
    } = options;
    if let Some(value) = path {
        if !value.exists() {
            eprintln!("=> {} Path not found: {}", Label::fail(), value.display());
            exit(exitcode::NOINPUT);
        }
    }
    if let Some(value) = reference {
        if !value.exists() {
            eprintln!("=> {} Reference not found: {}", Label::fail(), value.display());
            exit(exitcode::NOINPUT);
        }
    }
    if *offline {
        if !*quiet {
            println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
        }
        if !*supported {
            eprintln!("=> {} Offline mode is not implemented for '{command}' command", Label::fail());
            exit(exitcode::UNAVAILABLE);
        }
    }
}
pub async fn run(args: &Arguments, offline: bool, database_path: &Option<PathBuf>, threads: usize) -> Void {
    match &args.command {
        | Some(Commands::Check {
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
        }) => {
            Box::pin(check::run(
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
            ))
            .await
        }
        | Some(Commands::Create { command, .. }) => match command {
            | Some(CreateCommands::Mcp) => unimplemented!("MCP server creation is not yet implemented"),
            | Some(CreateCommands::Runner(args)) => {
                Box::pin(create::runner::run(
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
                ))
                .await
            }
            | None => {
                eprintln!("=> {} No subcommand provided for 'create'", Label::fail());
                exit(exitcode::USAGE);
            }
        },
        | Some(Commands::Doctor {
            fix,
            interactive,
            report,
            check,
            ..
        }) => Box::pin(doctor::run(fix, interactive, report, check, offline)).await,
        | Some(Commands::Download {
            config,
            url,
            filter,
            ignore,
            output,
            verbose,
            ..
        }) => {
            Box::pin(download::run(
                config,
                url,
                filter,
                ignore,
                output,
                database_path,
                threads,
                verbose,
                offline,
            ))
            .await
        }
        | Some(Commands::Export {
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
            verbose,
            ..
        }) => {
            Box::pin(export::run(
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
                threads,
                verbose,
                offline,
            ))
            .await
        }
        | Some(Commands::Format {
            path,
            branch,
            commit,
            filter,
            ignore,
            dry_run,
            merge_request,
            verbose,
            ..
        }) => {
            Box::pin(format::run(
                path,
                branch,
                commit,
                filter,
                ignore,
                *dry_run,
                merge_request,
                verbose,
                offline,
            ))
            .await
        }
        | Some(Commands::Gather {
            path,
            filter,
            ignore,
            merge_request,
            verbose,
            ..
        }) => Box::pin(gather::run(path, filter, ignore, merge_request, database_path, verbose, offline)).await,
        | Some(Commands::Link {
            path,
            branch,
            commit,
            ignore,
            dry_run,
            merge_request,
            verbose,
            ..
        }) => Box::pin(link::run(path, branch, commit, ignore, *dry_run, *merge_request, verbose)).await,
        | Some(Commands::Schema { command, .. }) => match command {
            | Some(SchemaCommands::Rad) | None => print_rad_schema(),
            | Some(SchemaCommands::Raid) => print_raid_schema(),
        },
        | Some(Commands::Tui) => Box::pin(tui::run()).await,
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
    exit(exitcode::OK);
}
fn print_raid_schema() -> Void {
    Metadata::to_schema();
    exit(exitcode::OK);
}
