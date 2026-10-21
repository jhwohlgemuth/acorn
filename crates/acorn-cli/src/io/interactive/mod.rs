//! Bounded inline terminal viewport for command-owned interaction.
use acorn::util::constants::app::{MAX_VIEWPORT_HEIGHT, MAX_VIEWPORT_HISTORY_LINES};
use acorn_core::prelude::Vec;
use color_eyre::eyre::{eyre, Result};
use crossterm::{
    cursor::{self, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    queue,
    style::{Attribute, Print, ResetColor, SetAttribute},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, IsTerminal, Write};
use unicode_width::UnicodeWidthChar;

/// One completed input operation from the inline viewport.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Input {
    Cancelled,
    Submitted(String),
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
    fn resize(&mut self, width: u16, rows: u16) {
        self.width = width;
        self.anchor = self.origin.min(rows.saturating_sub(1));
        self.height = rows.saturating_sub(self.anchor).clamp(1, MAX_VIEWPORT_HEIGHT);
    }
    fn render(&mut self) -> Result<()> {
        let mut output = io::stdout().lock();
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
fn display_width(text: &str) -> usize {
    text.chars().map(|character| UnicodeWidthChar::width(character).unwrap_or(0)).sum()
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
