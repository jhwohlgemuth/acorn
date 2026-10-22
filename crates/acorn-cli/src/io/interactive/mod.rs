//! Bounded inline terminal viewport for command-owned interaction.
use acorn::util::constants::app::{MAX_VIEWPORT_HEIGHT, MAX_VIEWPORT_HISTORY_LINES};
use acorn_core::prelude::Vec;
use color_eyre::eyre::{eyre, Result};
use core::{future::Future, time::Duration};
use crossterm::{
    cursor::{self, MoveTo, Show},
    event::{self, Event, EventStream, KeyCode, KeyEventKind, KeyModifiers},
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use futures::StreamExt;
use std::{
    env,
    io::{self, IsTerminal, Write},
};
use unicode_width::UnicodeWidthChar;

/// One completed input operation from the inline viewport.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Input {
    Cancelled,
    Submitted(String),
}
pub(crate) enum Overlay {
    Choose {
        prompt: String,
        options: Vec<String>,
        selected: usize,
        no_color: bool,
    },
    Confirm {
        prompt: String,
        yes: bool,
        no_color: bool,
    },
    Spin {
        label: String,
        frame: usize,
    },
}
/// A completed prompt value or an explicit user cancellation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PromptOutcome<T> {
    Cancelled,
    Selected(T),
}
#[derive(Clone, Copy)]
struct ConfirmButtonOptions<'a> {
    column: u16,
    focused: bool,
    no_color: bool,
    row: u16,
    text: &'a str,
    true_color: bool,
}
/// Terminal region owned by one interactive command.
#[derive(bon::Builder)]
#[builder(on(String, into))]
pub(crate) struct Viewport {
    anchor: u16,
    height: u16,
    #[builder(default)]
    history: Vec<String>,
    #[builder(default)]
    input: String,
    overlay: Option<Overlay>,
    origin: u16,
    #[builder(default)]
    scroll_from_bottom: usize,
    width: u16,
}
impl Drop for Viewport {
    fn drop(&mut self) {
        let mut output = io::stdout().lock();
        for offset in 0..self.height {
            let _ = queue!(output, MoveTo(0, self.anchor.saturating_add(offset)), Clear(ClearType::CurrentLine));
        }
        let _ = queue!(output, MoveTo(0, self.anchor), SetAttribute(Attribute::Reset), ResetColor, Show);
        let _ = output.flush();
        let _ = terminal::disable_raw_mode();
    }
}
impl Viewport {
    /// Claim a bounded region at the current cursor row.
    pub(crate) fn open() -> Result<Self> {
        (io::stdin().is_terminal() && io::stdout().is_terminal())
            .then_some(())
            .ok_or_else(|| eyre!("Interactive terminal input and output must both be TTYs"))
            .and_then(|()| cursor::position().map_err(Into::into))
            .and_then(|(_, row)| {
                terminal::size()
                    .map_err(Into::into)
                    .and_then(|(width, rows)| Self::open_at(row, width, rows))
            })
    }
    fn open_at(row: u16, width: u16, rows: u16) -> Result<Self> {
        (io::stdin().is_terminal() && io::stdout().is_terminal())
            .then_some(())
            .ok_or_else(|| eyre!("Interactive terminal input and output must both be TTYs"))
            .and_then(|()| terminal::enable_raw_mode().map_err(Into::into))
            .and_then(|()| {
                let rows = rows.max(1);
                let row = row.min(rows.saturating_sub(1));
                let desired_height = rows.min(MAX_VIEWPORT_HEIGHT);
                let scrolled = desired_height.saturating_sub(rows.saturating_sub(row));
                let anchor = row.saturating_sub(scrolled);
                let mut viewport = Self::builder().anchor(anchor).height(desired_height).origin(anchor).width(width).build();
                let prepared = {
                    let mut output = io::stdout().lock();
                    output
                        .write_all("\r\n".repeat(usize::from(desired_height.saturating_sub(1))).as_bytes())
                        .and_then(|()| output.flush())
                };
                prepared.map_err(Into::into).and_then(|()| {
                    viewport.resize(width, rows);
                    viewport.render().map(|()| viewport)
                })
            })
    }
    /// Add one complete output message to scrollable history.
    pub(crate) fn push(&mut self, text: impl Into<String>) -> Result<()> {
        self.history.push(text.into());
        if self.history.len() > MAX_VIEWPORT_HISTORY_LINES {
            self.history.remove(0);
        }
        self.scroll_from_bottom = 0;
        self.render()
    }
    /// Read and edit one line while retaining the viewport position.
    pub(crate) fn read_line(&mut self) -> Result<Input> {
        loop {
            let next = event::read().map_err(Into::into).and_then(|event| match event {
                | Event::Key(key) if key.kind == KeyEventKind::Press => match (key.code, key.modifiers) {
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => Ok(Some(Input::Cancelled)),
                    | (KeyCode::Enter, _) => {
                        let submitted = core::mem::take(&mut self.input);
                        self.render().map(|()| Some(Input::Submitted(submitted)))
                    }
                    | (KeyCode::Backspace, _) => {
                        self.input.pop();
                        self.render().map(|()| None)
                    }
                    | (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                        self.input.clear();
                        self.render().map(|()| None)
                    }
                    | (KeyCode::Char(character), modifiers) if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                        self.input.push(character);
                        self.render().map(|()| None)
                    }
                    | (KeyCode::Up, _) | (KeyCode::PageUp, _) => {
                        self.scroll_from_bottom = self.scroll_from_bottom.saturating_add(usize::from(self.height.saturating_sub(1).max(1)));
                        self.render().map(|()| None)
                    }
                    | (KeyCode::Down, _) | (KeyCode::PageDown, _) => {
                        self.scroll_from_bottom = self.scroll_from_bottom.saturating_sub(usize::from(self.height.saturating_sub(1).max(1)));
                        self.render().map(|()| None)
                    }
                    | _ => Ok(None),
                },
                | Event::Resize(width, rows) => {
                    self.resize(width, rows);
                    self.render().map(|()| None)
                }
                | Event::Paste(text) => {
                    self.input.push_str(&text.replace(['\r', '\n'], " "));
                    self.render().map(|()| None)
                }
                | _ => Ok(None),
            });
            match next {
                | Ok(Some(input)) => break Ok(input),
                | Err(why) => break Err(why),
                | Ok(None) => {}
            }
        }
    }
    /// Select one option by its index, or cancel without selecting.
    pub(crate) fn choose(&mut self, prompt: &str, options: &[String], no_color: bool) -> Result<PromptOutcome<usize>> {
        if options.is_empty() {
            Err(eyre!("Choose requires at least one option"))
        } else {
            self.choose_nonempty(prompt, options, no_color)
        }
    }
    fn choose_nonempty(&mut self, prompt: &str, options: &[String], no_color: bool) -> Result<PromptOutcome<usize>> {
        self.overlay = Some(Overlay::Choose {
            prompt: prompt.to_string(),
            options: options.to_vec(),
            selected: 0,
            no_color: no_color || env::var_os("NO_COLOR").is_some(),
        });
        let result = self.render().and_then(|()| loop {
            let outcome: Result<Option<PromptOutcome<usize>>> = match event::read() {
                | Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => match key.code {
                    | KeyCode::Esc => Ok(Some(PromptOutcome::Cancelled)),
                    | KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Ok(Some(PromptOutcome::Cancelled)),
                    | KeyCode::Enter => match &self.overlay {
                        | Some(Overlay::Choose { selected, .. }) => Ok(Some(PromptOutcome::Selected(*selected))),
                        | _ => Ok(None),
                    },
                    | KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(Overlay::Choose { selected, .. }) = &mut self.overlay {
                            *selected = selected.saturating_sub(1);
                        }
                        self.render().map(|()| None)
                    }
                    | KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(Overlay::Choose { selected, options, .. }) = &mut self.overlay {
                            *selected = selected.saturating_add(1).min(options.len().saturating_sub(1));
                        }
                        self.render().map(|()| None)
                    }
                    | _ => Ok(None),
                },
                | Ok(Event::Resize(width, rows)) => {
                    self.resize(width, rows);
                    self.render().map(|()| None)
                }
                | Ok(_) => Ok(None),
                | Err(why) => Err(why.into()),
            };
            match outcome {
                | Ok(Some(outcome)) => break Ok(outcome),
                | Err(why) => break Err(why),
                | Ok(None) => {}
            }
        });
        self.overlay = None;
        self.render().and(result)
    }
    /// Show separate Yes and No buttons; Enter defaults to No.
    pub(crate) fn confirm(&mut self, prompt: &str, no_color: bool) -> Result<PromptOutcome<bool>> {
        self.overlay = Some(Overlay::Confirm {
            prompt: prompt.to_string(),
            yes: false,
            no_color: no_color || env::var_os("NO_COLOR").is_some(),
        });
        let result = self.render().and_then(|()| loop {
            let outcome: Result<Option<PromptOutcome<bool>>> = match event::read() {
                | Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => match key.code {
                    | KeyCode::Esc => Ok(Some(PromptOutcome::Cancelled)),
                    | KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Ok(Some(PromptOutcome::Cancelled)),
                    | KeyCode::Char('y') => Ok(Some(PromptOutcome::Selected(true))),
                    | KeyCode::Char('n') => Ok(Some(PromptOutcome::Selected(false))),
                    | KeyCode::Enter => match &self.overlay {
                        | Some(Overlay::Confirm { yes, .. }) => Ok(Some(PromptOutcome::Selected(*yes))),
                        | _ => Ok(None),
                    },
                    | KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                        if let Some(Overlay::Confirm { yes, .. }) = &mut self.overlay {
                            *yes = !*yes;
                        }
                        self.render().map(|()| None)
                    }
                    | _ => Ok(None),
                },
                | Ok(Event::Resize(width, rows)) => {
                    self.resize(width, rows);
                    self.render().map(|()| None)
                }
                | Ok(_) => Ok(None),
                | Err(why) => Err(why.into()),
            };
            match outcome {
                | Ok(Some(outcome)) => break Ok(outcome),
                | Err(why) => break Err(why),
                | Ok(None) => {}
            }
        });
        self.overlay = None;
        self.render().and(result)
    }
    /// Animate within the viewport until a future completes, fails, or is cancelled.
    pub(crate) async fn spin<T, F>(&mut self, label: &str, operation: F) -> Result<PromptOutcome<T>>
    where
        F: Future<Output = Result<T>>,
    {
        self.overlay = Some(Overlay::Spin {
            label: label.to_string(),
            frame: 0,
        });
        let result = match self.render() {
            | Err(why) => Err(why),
            | Ok(()) => {
                let mut events = EventStream::new();
                let mut ticker = tokio::time::interval(Duration::from_millis(80));
                tokio::pin!(operation);
                loop {
                    tokio::select! {
                        biased;
                        value = &mut operation => break value.map(PromptOutcome::Selected),
                        next = events.next() => match next {
                            | Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press &&
                                (key.code == KeyCode::Esc || key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)) => {
                                break Ok(PromptOutcome::Cancelled)
                            }
                            | Some(Ok(Event::Resize(width, rows))) => {
                                self.resize(width, rows);
                                if let Err(why) = self.render() {
                                    break Err(why);
                                }
                            }
                            | Some(Err(why)) => break Err(why.into()),
                            | None => break Err(eyre!("Terminal event stream ended during spinner")),
                            | _ => {}
                        },
                        _ = ticker.tick() => {
                            if let Some(Overlay::Spin { frame, .. }) = &mut self.overlay {
                                *frame = frame.wrapping_add(1);
                            }
                            if let Err(why) = self.render() {
                                break Err(why);
                            }
                        }
                    }
                }
            }
        };
        self.overlay = None;
        self.render().and(result)
    }
    fn resize(&mut self, width: u16, rows: u16) {
        self.width = width;
        self.anchor = self.origin.min(rows.saturating_sub(1));
        self.height = rows.saturating_sub(self.anchor).clamp(1, MAX_VIEWPORT_HEIGHT);
    }
    fn render(&mut self) -> Result<()> {
        let mut output = io::stdout().lock();
        if self.overlay.is_some() {
            self.render_overlay(&mut output)
        } else {
            let usable_width = usize::from(self.width.saturating_sub(1));
            let lines = self
                .history
                .iter()
                .flat_map(|message| wrap_lines(message, usable_width.max(1)))
                .collect::<Vec<_>>();
            let history_rows = usize::from(self.height.saturating_sub(1));
            let max_scroll = lines.len().saturating_sub(history_rows);
            self.scroll_from_bottom = self.scroll_from_bottom.min(max_scroll);
            let end = lines.len().saturating_sub(self.scroll_from_bottom);
            let start = end.saturating_sub(history_rows);
            let history = (0..self.height).try_for_each(|offset| {
                queue!(output, MoveTo(0, self.anchor.saturating_add(offset)), Clear(ClearType::CurrentLine)).and_then(|()| {
                    match (
                        offset < self.height.saturating_sub(1),
                        lines.get(start.saturating_add(usize::from(offset))),
                    ) {
                        | (true, Some(line)) => queue!(output, Print(clip_to_width(line, usable_width))),
                        | _ => Ok(()),
                    }
                })
            });
            let input_row = self.anchor.saturating_add(self.height.saturating_sub(1));
            let input_width = usable_width.saturating_sub(2);
            let input = wrap_lines(&self.input, input_width.max(1)).into_iter().last().unwrap_or_default();
            let display = clip_to_width(&format!("> {input}"), usable_width);
            history
                .and_then(|()| {
                    queue!(
                        output,
                        MoveTo(0, input_row),
                        Print(&display),
                        MoveTo(display_width(&display).min(usable_width) as u16, input_row),
                        Show
                    )
                })
                .and_then(|()| output.flush())
                .map_err(Into::into)
        }
    }
    fn render_overlay(&self, output: &mut impl Write) -> Result<()> {
        let width = usize::from(self.width.saturating_sub(1));
        (0..self.height)
            .try_for_each(|offset| queue!(output, MoveTo(0, self.anchor.saturating_add(offset)), Clear(ClearType::CurrentLine)))
            .and_then(|()| match &self.overlay {
                | Some(Overlay::Choose {
                    prompt,
                    options,
                    selected,
                    no_color,
                }) => {
                    let visible = usize::from(self.height.saturating_sub(1));
                    let first = selected.saturating_sub(visible.saturating_sub(1));
                    print_overlay(output, 0, self.anchor, &clip_to_width(prompt, width), None).and_then(|()| {
                        options.iter().enumerate().skip(first).take(visible).try_for_each(|(index, option)| {
                            let marker = if index == *selected { "> " } else { "  " };
                            let line = clip_to_width(&format!("{marker}{option}"), width);
                            let offset = u16::try_from(index.saturating_sub(first).saturating_add(1)).unwrap_or(u16::MAX);
                            let color = (index == *selected && !*no_color).then_some(Color::Green);
                            print_overlay(output, 0, self.anchor.saturating_add(offset), &line, color)
                        })
                    })
                }
                | Some(Overlay::Confirm { prompt, yes, no_color }) => print_overlay(output, 0, self.anchor, &clip_to_width(prompt, width), None)
                    .and_then(|()| {
                        if self.height <= 1 {
                            Ok(())
                        } else {
                            let yes_button = if *yes { "> [Yes]" } else { "  [Yes]" };
                            let no_button = if *yes { "  [No]" } else { "> [No]" };
                            let button_row = self.anchor.saturating_add(1);
                            let true_color = env::var("COLORTERM")
                                .ok()
                                .is_some_and(|value| value.eq_ignore_ascii_case("truecolor") || value.eq_ignore_ascii_case("24bit"));
                            let button = ConfirmButtonOptions {
                                column: 0,
                                focused: *yes,
                                no_color: *no_color,
                                row: button_row,
                                text: yes_button,
                                true_color,
                            };
                            if width >= 16 {
                                print_confirm_button(output, button).and_then(|()| {
                                    let column = u16::try_from(yes_button.len().saturating_add(2)).unwrap_or(u16::MAX);
                                    let options = ConfirmButtonOptions {
                                        column,
                                        focused: !*yes,
                                        text: no_button,
                                        ..button
                                    };
                                    print_confirm_button(output, options)
                                })
                            } else if self.height > 2 {
                                let text = clip_to_width(yes_button, width);
                                print_confirm_button(output, ConfirmButtonOptions { text: &text, ..button }).and_then(|()| {
                                    let row = self.anchor.saturating_add(2);
                                    let text = clip_to_width(no_button, width);
                                    let options = ConfirmButtonOptions {
                                        focused: !*yes,
                                        row,
                                        text: &text,
                                        ..button
                                    };
                                    print_confirm_button(output, options)
                                })
                            } else {
                                let compact = if *yes { ">Y  N" } else { " Y >N" };
                                print_overlay(output, 0, button_row, &clip_to_width(compact, width), None)
                            }
                        }
                    }),
                | Some(Overlay::Spin { label, frame }) => {
                    let symbol = ["|", "/", "-", "\\"].get(frame % 4).copied().unwrap_or("|");
                    print_overlay(output, 0, self.anchor, &clip_to_width(&format!("{symbol} {label}"), width), None)
                }
                | None => Ok(()),
            })
            .and_then(|()| output.flush())
            .map_err(Into::into)
    }
}
fn clip_to_width(text: &str, width: usize) -> String {
    text.chars()
        .scan(0usize, |used, character| {
            let next = used.saturating_add(UnicodeWidthChar::width(character).unwrap_or(0));
            (next <= width).then(|| {
                *used = next;
                character
            })
        })
        .collect()
}
fn confirm_palette(options: ConfirmButtonOptions<'_>) -> (Color, Color) {
    match (options.focused, options.true_color) {
        | (true, true) => (Color::Rgb { r: 255, g: 70, b: 170 }, Color::Black),
        | (false, true) => (Color::Rgb { r: 58, g: 54, b: 70 }, Color::White),
        | (true, false) => (Color::Magenta, Color::White),
        | (false, false) => (Color::DarkGrey, Color::White),
    }
}
fn display_width(text: &str) -> usize {
    text.chars().map(|character| UnicodeWidthChar::width(character).unwrap_or(0)).sum()
}
fn print_confirm_button(output: &mut impl Write, options: ConfirmButtonOptions<'_>) -> io::Result<()> {
    match options.no_color {
        | true => queue!(output, MoveTo(options.column, options.row), Print(options.text)),
        | false => {
            let (background, foreground) = confirm_palette(options);
            queue!(
                output,
                MoveTo(options.column, options.row),
                SetBackgroundColor(background),
                SetForegroundColor(foreground),
                Print(options.text),
                ResetColor
            )
        }
    }
}
fn print_overlay(output: &mut impl Write, column: u16, row: u16, text: &str, color: Option<Color>) -> io::Result<()> {
    queue!(output, MoveTo(column, row)).and_then(|()| match color {
        | Some(color) => queue!(output, SetForegroundColor(color), Print(text), ResetColor),
        | None => queue!(output, Print(text)),
    })
}
fn wrap_lines(text: &str, width: usize) -> Vec<String> {
    text.split('\n')
        .flat_map(|paragraph| {
            let mut lines = vec![String::new()];
            let mut used = 0usize;
            for character in paragraph.chars() {
                let character = if character.is_control() { '�' } else { character };
                let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
                if used.saturating_add(character_width) > width && used > 0 {
                    lines.push(String::new());
                    used = 0;
                }
                if let Some(line) = lines.last_mut() {
                    line.push(if character_width > width { '?' } else { character });
                }
                used = used.saturating_add(character_width.min(width).max(usize::from(character_width > width)));
            }
            lines
        })
        .collect()
}
#[cfg(test)]
mod tests;
