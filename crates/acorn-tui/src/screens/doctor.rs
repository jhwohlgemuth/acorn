//! Interactive browser and confirmer for shared host diagnostic records.

use crate::app::App;
use crate::event::{to_internal_event, Event};
use crate::export;
use crate::Screen;
use acorn::analyzer::{
    fix::{CommandRunner, DiscoveryReference, HostProvider, IntoFixes, Plan, Policy, Provider, Verifier},
    host::discovery::{FindingStatus, Record, Records},
};
use core::time::Duration;
use crossterm::event;
use jiff::Timestamp;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use std::path::Path;
struct HostVerifier;
impl Verifier for HostVerifier {
    fn verify(&self, references: &[DiscoveryReference]) -> bool {
        let current = Records::diagnose_host();
        references.iter().all(|reference| {
            current.iter().any(|record| match record {
                | Record::Finding {
                    category,
                    status: FindingStatus::Healthy,
                    subject,
                    ..
                } => category == reference.category.as_ref() && subject == &reference.subject,
                | _ => false,
            })
        })
    }
}
fn apply_plan(app: &mut App) {
    let outcome = app.doctor.plan.as_ref().map_or_else(
        || Err("No trusted plan is available".to_string()),
        |plan| {
            let root = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
            plan.execute(&root, &CommandRunner::new(Duration::from_secs(120)), HostVerifier)
                .map(|outcomes| format!("Applied {} trusted operation(s)", outcomes.len()))
                .map_err(|why| why.to_string())
        },
    );
    app.refresh_doctor();
    app.doctor.export_message = Some(outcome.unwrap_or_else(|why| format!("Fix failed: {why}")));
}
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(area.height.saturating_sub(height) / 2),
            Constraint::Length(height.min(area.height)),
            Constraint::Min(0),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100_u16.saturating_sub(percent_x)) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Min(0),
        ])
        .split(vertical[1])[1]
}
fn decline_plan(app: &mut App) {
    app.doctor.confirming = false;
    app.doctor.plan = None;
    app.doctor.export_message = Some("Fix plan declined".to_string());
}
fn export(app: &mut App) {
    if let Some(data) = &app.doctor.data {
        let timestamp = Timestamp::now().strftime("%Y%m%d_%H%M%S");
        let filename = format!("acorn_doctor_report_{timestamp}.json");
        app.doctor.export_message = Some(match export::export_doctor_report(data, Some(Path::new(&filename))) {
            | Ok(path) => format!("Report saved to {path}"),
            | Err(why) => format!("Export failed: {why}"),
        });
    }
}
pub fn handle_event(app: &mut App, event: event::Event) {
    let Some(event) = (match event {
        | event::Event::Key(key) => to_internal_event(key),
        | _ => None,
    }) else {
        return;
    };
    if app.doctor.confirming {
        match event {
            | Event::Char('y' | 'Y') => apply_plan(app),
            | Event::Char('n' | 'N') | Event::Back => decline_plan(app),
            | Event::Quit => app.should_quit = true,
            | _ => {}
        }
        return;
    }
    match event {
        | Event::Quit => app.should_quit = true,
        | Event::Back => app.navigate_to(Screen::Dashboard),
        | Event::Up => app.doctor.selected_record = app.doctor.selected_record.saturating_sub(1),
        | Event::Down => {
            let maximum = app.doctor.data.as_ref().map_or(0, |records| records.len().saturating_sub(1));
            app.doctor.selected_record = app.doctor.selected_record.saturating_add(1).min(maximum);
        }
        | Event::Refresh => app.refresh_doctor(),
        | Event::Export => export(app),
        | Event::Char(' ') if app.options.doctor_fix => toggle(app),
        | Event::Char('f' | 'F') if app.options.doctor_fix => preview(app),
        | _ => {}
    }
}
fn preview(app: &mut App) {
    let references = app.doctor.data.as_ref().map_or_else(Vec::new, |records| {
        records
            .fix_references()
            .into_iter()
            .enumerate()
            .filter(|(index, _)| app.doctor.selected.contains(index))
            .map(|(_, reference)| reference)
            .collect()
    });
    let report = HostProvider.provide(&references);
    let fixes = report.fixes().into_iter().cloned().collect::<Vec<_>>();
    match Plan::from_fixes(&fixes, Policy::confirmed(app.options.offline)) {
        | Ok(plan) if !plan.preview().is_empty() => {
            app.doctor.plan = Some(plan);
            app.doctor.confirming = true;
        }
        | Ok(_) => app.doctor.export_message = Some("No trusted fixes are available for the selection".to_string()),
        | Err(why) => app.doctor.export_message = Some(format!("Cannot plan fixes: {why}")),
    }
}
pub fn render(frame: &mut Frame, app: &mut App) {
    app.load_doctor_data();
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(chunks[0]);
    render_findings(frame, main[0], app);
    render_details(frame, main[1], app);
    render_status(frame, chunks[1], app);
    if app.doctor.confirming {
        render_confirmation(frame, centered_rect(70, 10, area), app);
    }
}
fn render_confirmation(frame: &mut Frame, area: Rect, app: &App) {
    let lines = app
        .doctor
        .plan
        .as_ref()
        .map(|plan| {
            plan.preview()
                .iter()
                .map(|operation| Line::from(format!(" • {operation}")))
                .chain([Line::from(""), Line::from(" Apply these fixed operations? [Y] yes / [N or Esc] no")])
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![Line::from(" No trusted operations selected")]);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Confirm fixes ")
                    .border_style(Style::default().fg(app.theme.warning)),
            )
            .alignment(Alignment::Left),
        area,
    );
}
fn render_details(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = app
        .doctor
        .data
        .as_ref()
        .and_then(|records| records.iter().nth(app.doctor.selected_record))
        .map_or_else(
            || vec![Line::from(" No finding selected")],
            |record| match record {
                | Record::Finding {
                    category,
                    details,
                    expected,
                    observed,
                    status,
                    subject,
                } => core::iter::once(Line::from(format!(" Subject:  {subject}")))
                    .chain([
                        Line::from(format!(" Category: {category}")),
                        Line::from(format!(" Status:   {status}")),
                        Line::from(format!(" Observed: {observed}")),
                        Line::from(format!(" Expected: {expected}")),
                    ])
                    .chain(details.iter().map(|(key, value)| Line::from(format!(" {key}: {value}"))))
                    .collect(),
                | _ => vec![Line::from(" Unsupported diagnostic record")],
            },
        );
    if let Some(message) = &app.doctor.export_message {
        lines.extend([Line::from(""), Line::from(format!(" {message}"))]);
    }
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Details ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(app.theme.accent)),
        ),
        area,
    );
}
fn render_findings(frame: &mut Frame, area: Rect, app: &App) {
    let lines = app.doctor.data.as_ref().map_or_else(
        || vec![Line::from(" Loading diagnostics...")],
        |records| {
            records
                .iter()
                .enumerate()
                .filter_map(|(index, record)| match record {
                    | Record::Finding {
                        category, status, subject, ..
                    } => {
                        let cursor = if index == app.doctor.selected_record { ">" } else { " " };
                        let selected = if app.doctor.selected.contains(&index) { "x" } else { " " };
                        let status = match status {
                            | FindingStatus::Healthy => "✓",
                            | FindingStatus::Attention => "!",
                            | FindingStatus::Unavailable => "?",
                        };
                        let style = if index == app.doctor.selected_record {
                            Style::default()
                                .fg(app.theme.selection_fg)
                                .bg(app.theme.selection_bg)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(app.theme.text)
                        };
                        Some(Line::from(Span::styled(
                            format!(" {cursor} [{selected}] {status} {category}: {subject}"),
                            style,
                        )))
                    }
                    | _ => None,
                })
                .collect()
        },
    );
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Host findings ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(app.theme.accent)),
        ),
        area,
    );
}
fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let mut controls = vec![
        Span::styled(" [↑/↓] Navigate  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[R] Refresh  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[E] Export  ", Style::default().fg(app.theme.text_muted)),
    ];
    if app.options.doctor_fix {
        controls.extend([
            Span::styled("[Space] Toggle  ", Style::default().fg(app.theme.text_muted)),
            Span::styled("[F] Preview  ", Style::default().fg(app.theme.text_muted)),
        ]);
    }
    controls.push(Span::styled("[Esc] Back  [Q] Quit", Style::default().fg(app.theme.text_muted)));
    frame.render_widget(
        Paragraph::new(Line::from(controls))
            .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(app.theme.border)))
            .alignment(Alignment::Center),
        area,
    );
}
fn toggle(app: &mut App) {
    let index = app.doctor.selected_record;
    if !app.doctor.selected.insert(index) {
        app.doctor.selected.remove(&index);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::TuiOptions;

    #[test]
    fn decline_clears_confirmation_without_applying() {
        let mut app = App::with_options(
            Screen::Doctor,
            TuiOptions {
                doctor_fix: true,
                ..TuiOptions::default()
            },
        );
        app.doctor.confirming = true;
        decline_plan(&mut app);
        assert!(!app.doctor.confirming);
        assert!(app.doctor.plan.is_none());
    }
    #[test]
    fn selection_toggles_current_finding() {
        let mut app = App::with_options(
            Screen::Doctor,
            TuiOptions {
                doctor_fix: true,
                ..TuiOptions::default()
            },
        );
        toggle(&mut app);
        assert!(app.doctor.selected.contains(&0));
        toggle(&mut app);
        assert!(app.doctor.selected.is_empty());
    }
}
