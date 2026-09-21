use crate::cli::{WatchMode, WatchOptions};
use acorn::{
    io::{directory_roots, Fingerprint},
    util::constants::app::WATCH_DEBOUNCE_INTERVAL,
};
use acorn_core::prelude::Vec;
use acorn_host::terminal::Label;
use color_eyre::eyre::{eyre, Report, Result};
use core::future::{pending, Future};
use crossterm::{
    event::{Event as TerminalEvent, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use notify::{PollWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer_opt, Config, DebounceEventResult, Debouncer};
use owo_colors::OwoColorize;
use std::{
    io::{stdin, IsTerminal},
    path::PathBuf,
};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
    time::sleep,
};

pub(crate) async fn run<T, Prepare, PrepareFuture, Execute, ExecuteFuture>(
    command: &str,
    roots: &[PathBuf],
    options: WatchOptions,
    mut prepare: Prepare,
    mut execute: Execute,
) -> Result<()>
where
    Prepare: FnMut() -> PrepareFuture,
    PrepareFuture: Future<Output = Result<Prepared<T>>>,
    Execute: FnMut(T) -> ExecuteFuture,
    ExecuteFuture: Future<Output = Result<()>>,
{
    match Trigger::new(roots, options) {
        | Ok(trigger) => run_with_trigger(command, trigger, &mut prepare, &mut execute).await,
        | Err(why) => Err(why),
    }
}
async fn run_with_trigger<T, Prepare, PrepareFuture, Execute, ExecuteFuture>(
    command: &str,
    mut trigger: Trigger,
    prepare: &mut Prepare,
    execute: &mut Execute,
) -> Result<()>
where
    Prepare: FnMut() -> PrepareFuture,
    PrepareFuture: Future<Output = Result<Prepared<T>>>,
    Execute: FnMut(T) -> ExecuteFuture,
    ExecuteFuture: Future<Output = Result<()>>,
{
    let mut quit = QuitEvents::new();
    let mut previous = None;
    let mut first = true;
    loop {
        match prepare().await {
            | Ok(prepared) if previous.as_ref() == Some(&prepared.fingerprint) => {}
            | Ok(prepared) => {
                if !first {
                    let timestamp = jiff::Timestamp::now().strftime("%H:%M:%S");
                    eprintln!("{} — re-running {} at {timestamp}", Label::changed(), format!("acorn {command}").cyan());
                }
                first = false;
                previous = Some(prepared.fingerprint);
                if let Err(why) = execute(prepared.value).await {
                    eprintln!("Error: {why:?}");
                }
            }
            | Err(why) => eprintln!("Error: {why:?}"),
        }
        match tokio::select! {
            result = trigger.changed() => Event::Changed(result),
            result = quit.stopped() => Event::Stopped(result),
            signal = tokio::signal::ctrl_c() => Event::Stopped(signal.map_err(Report::from)),
        } {
            | Event::Changed(Ok(next)) => trigger = next,
            | Event::Stopped(Ok(())) => break Ok(()),
            | Event::Changed(Err(why)) | Event::Stopped(Err(why)) => break Err(why),
        }
    }
}
enum Guard {
    Native { watcher: Debouncer<RecommendedWatcher> },
    Poll { watcher: Debouncer<PollWatcher> },
    Timer,
}
enum Event {
    Changed(Result<Trigger>),
    Stopped(Result<()>),
}
struct QuitEvents {
    events: Option<EventStream>,
}
struct RawModeGuard;
/// Prepared command input paired with its stable content fingerprint.
pub(crate) struct Prepared<T> {
    pub(crate) value: T,
    pub(crate) fingerprint: Fingerprint,
}
struct Trigger {
    guard: Guard,
    receiver: UnboundedReceiver<DebounceEventResult>,
    roots: Vec<PathBuf>,
    options: WatchOptions,
}
impl Guard {
    fn is_native(&self) -> bool {
        match self {
            | Self::Native { watcher } => {
                let _ = watcher;
                true
            }
            | Self::Poll { watcher } => {
                let _ = watcher;
                false
            }
            | Self::Timer => false,
        }
    }
}
impl<T> Prepared<T> {
    pub(crate) const fn new(value: T, fingerprint: Fingerprint) -> Self {
        Self { value, fingerprint }
    }
}
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}
impl QuitEvents {
    fn new() -> Self {
        let events = match stdin().is_terminal() {
            | true => Some(EventStream::new()),
            | false => None,
        };
        Self { events }
    }
    async fn stopped(&mut self) -> Result<()> {
        match self.events.as_mut() {
            | Some(events) => match enable_raw_mode() {
                | Ok(()) => {
                    let _raw_mode = RawModeGuard;
                    loop {
                        match events.next().await {
                            | Some(Ok(TerminalEvent::Key(key))) if is_quit_key(key) => break Ok(()),
                            | Some(Ok(_)) => {}
                            | Some(Err(why)) => break Err(Report::from(why)),
                            | None => break pending().await,
                        }
                    }
                }
                | Err(_) => pending().await,
            },
            | None => pending().await,
        }
    }
}
impl Trigger {
    fn new(paths: &[PathBuf], options: WatchOptions) -> Result<Self> {
        let roots = directory_roots(paths);
        match (roots.is_empty(), options.network, options.mode) {
            | (true, false, _) => Err(eyre!("No dynamic sources are available to watch")),
            | (_, true, WatchMode::Native) => Err(eyre!("Native watch mode cannot watch network sources")),
            | (_, true, _) | (_, _, WatchMode::Poll) => Self::poll(roots, options),
            | (_, false, WatchMode::Native) => Self::native(roots, options),
            | (_, false, WatchMode::Auto) => Self::native(roots.clone(), options).or_else(|_| Self::poll(roots, options)),
        }
    }
    pub(crate) async fn changed(self) -> Result<Self> {
        let Self {
            guard,
            mut receiver,
            roots,
            options,
        } = self;
        let result = match (&guard, options.network) {
            | (Guard::Timer, _) => {
                sleep(options.poll_interval.unsigned_abs()).await;
                Ok(())
            }
            | (_, true) => {
                tokio::select! {
                    event = receiver.recv() => match event {
                        | Some(event) => event.map(|_| ()).map_err(Report::from),
                        | None => {
                            sleep(options.poll_interval.unsigned_abs()).await;
                            Ok(())
                        },
                    },
                    () = sleep(options.poll_interval.unsigned_abs()) => Ok(()),
                }
            }
            | (_, false) => receiver.recv().await.map_or_else(
                || Err(eyre!("Watch event channel closed")),
                |event| event.map(|_| ()).map_err(Report::from),
            ),
        };
        match (guard.is_native(), result, options.mode) {
            | (true, Err(_), WatchMode::Auto) => Self::poll(roots, options),
            | (_, Ok(()), _) => Ok(Self {
                guard,
                receiver,
                roots,
                options,
            }),
            | (_, Err(why), _) => Err(why),
        }
    }
    fn native(roots: Vec<PathBuf>, options: WatchOptions) -> Result<Self> {
        let (sender, receiver) = unbounded_channel();
        let config = Config::default().with_timeout(WATCH_DEBOUNCE_INTERVAL.unsigned_abs());
        new_debouncer_opt::<_, RecommendedWatcher>(config, move |event| {
            let _ = sender.send(event);
        })
        .map_err(Report::from)
        .and_then(|mut watcher| {
            watch(&mut watcher, &roots).map(|()| Self {
                guard: Guard::Native { watcher },
                receiver,
                roots,
                options,
            })
        })
    }
    fn poll(roots: Vec<PathBuf>, options: WatchOptions) -> Result<Self> {
        let (sender, receiver) = unbounded_channel();
        let notify_config = notify::Config::default()
            .with_poll_interval(options.poll_interval.unsigned_abs())
            .with_compare_contents(true);
        let config = Config::default()
            .with_timeout(WATCH_DEBOUNCE_INTERVAL.unsigned_abs())
            .with_notify_config(notify_config);
        match roots.is_empty() {
            | true => Ok(Self {
                guard: Guard::Timer,
                receiver,
                roots,
                options,
            }),
            | false => new_debouncer_opt::<_, PollWatcher>(config, move |event| {
                let _ = sender.send(event);
            })
            .map_err(Report::from)
            .and_then(|mut watcher| {
                watch(&mut watcher, &roots).map(|()| Self {
                    guard: Guard::Poll { watcher },
                    receiver,
                    roots,
                    options,
                })
            }),
        }
    }
}
fn is_quit_key(key: KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
        && matches!((key.code, key.modifiers), |(KeyCode::Char('q'), KeyModifiers::NONE)| (
            KeyCode::Char('Q'),
            KeyModifiers::SHIFT
        ) | (
            KeyCode::Char('c' | 'C'),
            KeyModifiers::CONTROL
        ))
}
fn watch<T: Watcher>(watcher: &mut Debouncer<T>, roots: &[PathBuf]) -> Result<()> {
    roots.iter().try_for_each(|root| {
        let recursive = match root.is_dir() {
            | true => RecursiveMode::Recursive,
            | false => RecursiveMode::NonRecursive,
        };
        watcher.watcher().watch(root, recursive).map_err(Report::from)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;
    use std::{
        env::temp_dir,
        fs::{create_dir_all, remove_dir_all, write},
    };
    use tokio::time::timeout;

    #[test]
    fn q_and_control_c_stop_watch_mode() {
        assert!(is_quit_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)));
        assert!(is_quit_key(KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT)));
        assert!(is_quit_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
        assert!(!is_quit_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)));
    }
    #[tokio::test]
    async fn remote_timer_waits_for_the_poll_interval() {
        let options = WatchOptions {
            mode: WatchMode::Poll,
            poll_interval: SignedDuration::from_millis(50),
            network: true,
        };
        let trigger = Trigger::new(&[], options).expect("create remote timer");
        assert!(timeout(SignedDuration::from_millis(10).unsigned_abs(), trigger.changed()).await.is_err());
        let trigger = Trigger::new(&[], options).expect("create remote timer");
        assert!(timeout(SignedDuration::from_millis(100).unsigned_abs(), trigger.changed()).await.is_ok());
    }
    #[test]
    fn static_sources_cannot_start_a_watcher() {
        let options = WatchOptions {
            mode: WatchMode::Auto,
            poll_interval: SignedDuration::from_millis(50),
            network: false,
        };
        let result = Trigger::new(&[], options);
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn poll_watcher_reports_local_changes() {
        let root = temp_dir().join(format!("acorn-watch-{}", nanoid::nanoid!()));
        create_dir_all(&root).expect("create watch fixture");
        let options = WatchOptions {
            mode: WatchMode::Poll,
            poll_interval: SignedDuration::from_millis(50),
            network: false,
        };
        let trigger = Trigger::new(core::slice::from_ref(&root), options).expect("create polling watcher");
        write(root.join("input.json"), b"{}\n").expect("write watched fixture");
        let changed = timeout(SignedDuration::from_secs(2).unsigned_abs(), trigger.changed()).await;
        let _ = remove_dir_all(root);
        assert!(changed.is_ok(), "polling watcher did not report the change");
    }
}
