use super::*;
#[cfg(unix)]
use core::time::Duration;
#[cfg(unix)]
use nix::pty::{openpty, Winsize};
#[cfg(unix)]
use std::{
    fs::File,
    io::{Read, Write},
    process::{Command, Stdio},
    thread,
    time::Instant,
};

#[test]
fn test_clip_preserves_complete_wide_characters() {
    assert_eq!(clip_to_width("ab界cd", 4), "ab界");
    assert_eq!(clip_to_width("ab界cd", 3), "ab");
}
#[test]
fn test_confirm_button_palettes() {
    let options = |focused, no_color, true_color| ConfirmButtonOptions {
        column: 0,
        focused,
        no_color,
        row: 0,
        text: "> [Yes]",
        true_color,
    };
    let render = |focused, no_color, true_color| {
        let mut output = Vec::new();
        print_confirm_button(&mut output, options(focused, no_color, true_color)).expect("render button");
        String::from_utf8(output).expect("terminal bytes")
    };
    assert_eq!(
        confirm_palette(options(true, false, true)),
        (Color::Rgb { r: 255, g: 70, b: 170 }, Color::Black)
    );
    assert_eq!(
        confirm_palette(options(false, false, true)),
        (Color::Rgb { r: 58, g: 54, b: 70 }, Color::White)
    );
    assert_eq!(confirm_palette(options(true, false, false)), (Color::Magenta, Color::White));
    assert_eq!(confirm_palette(options(false, false, false)), (Color::DarkGrey, Color::White));
    assert!(!render(true, true, true).contains("[48;"));
}
#[test]
fn test_confirm_no_color_keeps_both_buttons_and_focus() {
    let render = |no_color| {
        let viewport = core::mem::ManuallyDrop::new(
            Viewport::builder()
                .anchor(0)
                .height(3)
                .origin(0)
                .width(24)
                .overlay(Overlay::Confirm {
                    prompt: "Apply?".into(),
                    yes: false,
                    no_color,
                })
                .build(),
        );
        let mut output = Vec::new();
        viewport.render_overlay(&mut output).expect("render buttons");
        String::from_utf8(output).expect("terminal bytes")
    };
    let colored = render(false);
    let plain = render(true);
    assert!(colored.contains("[Yes]") && colored.contains("> [No]"));
    assert!(plain.contains("[Yes]") && plain.contains("> [No]"));
    if env::var_os("NO_COLOR").is_none() {
        assert!(colored.contains("[45m") || colored.contains("[48;"), "{colored:?}");
    }
    assert!(!plain.contains("[48;"));
}
#[test]
fn test_non_tty_is_rejected_before_raw_mode() {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        assert!(Viewport::open_at(0, 40, 12).is_err());
    }
}
#[cfg(unix)]
#[test]
fn test_viewport_pty_child() {
    if std::env::var_os("ACORN_VIEWPORT_CHILD").is_none() {
        return;
    }
    crossterm::execute!(io::stdout(), MoveTo(0, 3)).expect("position command row");
    let mut viewport = Viewport::open_at(3, 24, 20).expect("PTY viewport");
    viewport.push("A long line that wraps inside the viewport").expect("draw history");
    viewport.resize(12, 7);
    viewport.render().expect("draw resized region");
    assert_eq!(viewport.read_line().expect("first input"), Input::Submitted("ok".to_string()));
    assert_eq!(viewport.read_line().expect("cancellation"), Input::Cancelled);
    assert_eq!(viewport.read_line().expect("interruption"), Input::Cancelled);
    assert_eq!(
        viewport
            .choose("Choose standard", &["RAD".to_string(), "DataCite".to_string()], false)
            .expect("choose"),
        PromptOutcome::Selected(1)
    );
    assert_eq!(
        viewport.confirm("Narrow?", false).expect("narrow refusal"),
        PromptOutcome::Selected(false)
    );
    viewport.resize(24, 20);
    viewport.render().expect("restore wide viewport");
    assert_eq!(
        viewport.confirm("Apply change?", false).expect("default refusal"),
        PromptOutcome::Selected(false)
    );
    assert_eq!(viewport.confirm("Apply change?", true).expect("approval"), PromptOutcome::Selected(true));
    assert_eq!(viewport.confirm("Apply change?", true).expect("cancellation"), PromptOutcome::Cancelled);
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("runtime");
    assert_eq!(
        runtime
            .block_on(viewport.spin("Working", async { Ok::<_, color_eyre::Report>(7) }))
            .expect("complete spin"),
        PromptOutcome::Selected(7)
    );
    assert!(runtime
        .block_on(viewport.spin("Failing", async { Err::<(), _>(eyre!("operation failed")) }))
        .is_err());
    assert_eq!(
        runtime
            .block_on(viewport.spin("Waiting", core::future::pending::<color_eyre::eyre::Result<()>>()))
            .expect("cancel spin"),
        PromptOutcome::Cancelled
    );
    drop(viewport);
    assert!(!terminal::is_raw_mode_enabled().expect("raw mode query"));
    crossterm::execute!(io::stdout(), MoveTo(0, 19)).expect("position bottom row");
    let mut bottom = Viewport::open_at(19, 24, 20).expect("bottom-row viewport");
    bottom.push("BOTTOM_VIEWPORT").expect("draw bottom-row history");
    drop(bottom);
    println!("AFTER_VIEWPORT");
}
#[cfg(unix)]
#[test]
fn test_viewport_stays_inline_and_restores_terminal_in_pty() {
    let pty = openpty(
        Some(&Winsize {
            ws_row: 20,
            ws_col: 24,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }),
        None,
    )
    .expect("PTY pair");
    let mut master: File = pty.master.into();
    let mut reader = master.try_clone().expect("PTY reader");
    let output = thread::spawn(move || {
        let mut rendered = Vec::new();
        let mut chunk = [0_u8; 4096];
        loop {
            match reader.read(&mut chunk) {
                | Ok(0) | Err(_) => break,
                | Ok(count) => rendered.extend_from_slice(chunk.get(..count).expect("read count should fit in the buffer")),
            }
        }
        String::from_utf8_lossy(&rendered).into_owned()
    });
    let slave: File = pty.slave.into();
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", "io::interactive::tests::test_viewport_pty_child", "--nocapture"])
        .env("ACORN_VIEWPORT_CHILD", "1")
        .stdin(Stdio::from(slave.try_clone().expect("stdin clone")))
        .stdout(Stdio::from(slave.try_clone().expect("stdout clone")))
        .stderr(Stdio::null())
        .spawn()
        .expect("PTY child");
    drop(slave);
    thread::sleep(Duration::from_millis(200));
    master.write_all(b"ok\r").expect("send input");
    thread::sleep(Duration::from_millis(100));
    master.write_all(b"\x1b").expect("send Escape");
    thread::sleep(Duration::from_millis(200));
    master.write_all(b"\x03").expect("send Ctrl-C");
    thread::sleep(Duration::from_millis(200));
    master.write_all(b"\x1b[B\r").expect("choose second option");
    thread::sleep(Duration::from_millis(200));
    master.write_all(b"\r").expect("confirm narrow default No");
    thread::sleep(Duration::from_millis(200));
    master.write_all(b"\r").expect("confirm default No");
    thread::sleep(Duration::from_millis(200));
    master.write_all(b"\x1b[C\r").expect("confirm Yes");
    thread::sleep(Duration::from_millis(200));
    master.write_all(b"\x1b").expect("cancel confirm");
    thread::sleep(Duration::from_millis(250));
    master.write_all(b"\x1b").expect("cancel spinner");
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait().expect("child status") {
            break Some(status);
        }
        if Instant::now() >= deadline {
            child.kill().expect("stop stalled child");
            break None;
        }
        thread::sleep(Duration::from_millis(20));
    };
    let status = status.expect("PTY viewport child did not finish");
    drop(master);
    let rendered = output.join().expect("PTY output reader");
    assert!(status.success(), "PTY child failed: {rendered}");
    assert!(rendered.contains("[4;1H"), "viewport must start on row four: {rendered:?}");
    assert!(rendered.contains("A long line"), "history was not rendered: {rendered:?}");
    assert!(
        rendered.contains("Choose stan") && rendered.contains("DataCite"),
        "choose control missing: {rendered:?}"
    );
    assert!(
        rendered.contains("[Yes]") && rendered.contains("[No]"),
        "confirm buttons missing: {rendered:?}"
    );
    assert!(
        rendered.contains("Working") && rendered.contains("Failing") && rendered.contains("Waiting"),
        "spinner missing: {rendered:?}"
    );
    assert!(
        rendered.contains("[9;1H") && rendered.contains("BOTTOM_VIEWPORT"),
        "bottom-row viewport did not reserve space: {rendered:?}"
    );
    assert!(rendered.contains("AFTER_VIEWPORT"), "guard did not restore terminal: {rendered:?}");
}
#[test]
fn test_wraps_long_text_and_narrow_widths() {
    assert_eq!(wrap_lines("abcdefgh", 3), vec!["abc", "def", "gh"]);
    assert_eq!(wrap_lines("one\ntwo", 1), vec!["o", "n", "e", "t", "w", "o"]);
    assert_eq!(wrap_lines("A\r\x1b[B", 20), vec!["A��[B"]);
    assert_eq!(wrap_lines("", 1), vec![""]);
}
