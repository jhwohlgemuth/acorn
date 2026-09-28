use super::*;
use jiff::SignedDuration;
use std::{
    env::temp_dir,
    fs::{create_dir_all, remove_dir_all, write},
};
use tokio::time::timeout;

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
