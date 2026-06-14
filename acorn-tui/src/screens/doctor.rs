use crate::app::App;
use crate::event::{to_internal_event, Event};
use crate::export;
use crate::Screen;
use crossterm::event;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const CATEGORIES: &[&str] = &["System", "Memory", "Network", "Software", "GPU"];

pub fn render(f: &mut Frame, app: &mut App) {
    app.load_doctor_data();
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(1)])
        .split(chunks[0]);
    render_category_list(f, main_chunks[0], app);
    render_detail_pane(f, main_chunks[1], app);
    render_status_bar(f, chunks[1], app);
}
fn render_category_list(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<Line> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_selected = i == app.doctor.selected_category;
            let prefix = if is_selected { ">" } else { " " };
            let style = if is_selected {
                Style::default()
                    .fg(app.theme.selection_fg)
                    .bg(app.theme.selection_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.text)
            };
            Line::from(Span::styled(format!(" {prefix} [{i}] {name}"), style))
        })
        .collect();
    let paragraph = Paragraph::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Diagnostics ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(app.theme.accent)),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}
fn render_detail_pane(f: &mut Frame, area: Rect, app: &App) {
    let Some(ref data) = app.doctor.data else {
        let paragraph = Paragraph::new(" Loading diagnostics...")
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(app.theme.border)))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    };
    let lines: Vec<Line> = match app.doctor.selected_category {
        | 0 => render_system(data, app),
        | 1 => render_memory(data, app),
        | 2 => render_network(data, app),
        | 3 => render_software(data, app),
        | 4 => render_gpu(app),
        | _ => vec![Line::from(" Unknown category")],
    };
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", CATEGORIES[app.doctor.selected_category]))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(app.theme.accent)),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, area);
    if let Some(ref msg) = app.doctor.export_message {
        let popup_area = centered_rect(50, 3, area);
        let popup = Paragraph::new(msg.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Export ")
                    .border_style(Style::default().fg(app.theme.accent)),
            )
            .alignment(Alignment::Center);
        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);
    }
}
fn render_system(data: &crate::app::DoctorData, app: &App) -> Vec<Line<'static>> {
    let Some(ref sys) = data.system else {
        return vec![Line::from(" No system data available")];
    };
    vec![
        Line::from(vec![
            Span::styled(" Name:             ", Style::default().fg(app.theme.text_muted)),
            Span::styled(sys.name.clone(), Style::default().fg(app.theme.text)),
        ]),
        Line::from(vec![
            Span::styled(" Kernel Version:   ", Style::default().fg(app.theme.text_muted)),
            Span::styled(sys.kernel.clone(), Style::default().fg(app.theme.text)),
        ]),
        Line::from(vec![
            Span::styled(" OS Version:       ", Style::default().fg(app.theme.text_muted)),
            Span::styled(sys.os_version.clone(), Style::default().fg(app.theme.text)),
        ]),
        Line::from(vec![
            Span::styled(" Host Name:        ", Style::default().fg(app.theme.text_muted)),
            Span::styled(sys.host_name.clone(), Style::default().fg(app.theme.text)),
        ]),
        Line::from(vec![
            Span::styled(" CPU Architecture: ", Style::default().fg(app.theme.text_muted)),
            Span::styled(sys.cpu_arch.clone(), Style::default().fg(app.theme.text)),
        ]),
        Line::from(vec![
            Span::styled(" CPU Count:        ", Style::default().fg(app.theme.text_muted)),
            Span::styled(sys.cpu_count.clone(), Style::default().fg(app.theme.text)),
        ]),
    ]
}
fn render_memory(data: &crate::app::DoctorData, app: &App) -> Vec<Line<'static>> {
    let Some(ref mem) = data.memory else {
        return vec![Line::from(" No memory data available")];
    };
    vec![
        Line::from(vec![
            Span::styled(" Total:     ", Style::default().fg(app.theme.text_muted)),
            Span::styled(mem.total.clone(), Style::default().fg(app.theme.text)),
        ]),
        Line::from(vec![
            Span::styled(" Available: ", Style::default().fg(app.theme.text_muted)),
            Span::styled(mem.available.clone(), Style::default().fg(app.theme.success)),
        ]),
        Line::from(vec![
            Span::styled(" Used:      ", Style::default().fg(app.theme.text_muted)),
            Span::styled(mem.used.clone(), Style::default().fg(app.theme.warning)),
        ]),
        Line::from(vec![
            Span::styled(" Swap:      ", Style::default().fg(app.theme.text_muted)),
            Span::styled(mem.swap.clone(), Style::default().fg(app.theme.text)),
        ]),
    ]
}
fn render_network(data: &crate::app::DoctorData, app: &App) -> Vec<Line<'static>> {
    let Some(ref net) = data.network else {
        return vec![Line::from(" No network data available")];
    };
    if net.interfaces.is_empty() {
        return vec![Line::from(" No active network interfaces found")];
    }
    let mut lines = Vec::new();
    for (i, iface) in net.interfaces.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            format!(" Interface {}:", i + 1),
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD),
        )));
        for ip in &iface.ip_addresses {
            lines.push(Line::from(Span::styled(format!("   IP:  {ip}"), Style::default().fg(app.theme.text))));
        }
        lines.push(Line::from(Span::styled(
            format!("   MAC: {}", iface.mac_address),
            Style::default().fg(app.theme.text),
        )));
        lines.push(Line::from(Span::styled(
            format!("   MTU: {}", iface.mtu),
            Style::default().fg(app.theme.text),
        )));
    }
    lines
}
fn render_software(data: &crate::app::DoctorData, app: &App) -> Vec<Line<'static>> {
    let Some(ref sw) = data.software else {
        return vec![Line::from(" No software data available")];
    };
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!(" {:<12}", "Name"),
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {:>9}", "Installed"),
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {:>12}", "Version"),
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {}", "Path"),
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD),
        ),
    ])];
    lines.push(Line::from(Span::styled("-".repeat(60), Style::default().fg(app.theme.text_muted))));
    for item in &sw.items {
        let status = if item.installed { "✓" } else { "✗" };
        let status_color = if item.installed { app.theme.success } else { app.theme.error };
        let path_display = if item.path.len() > 30 {
            format!("...{}", &item.path[item.path.len().saturating_sub(27)..])
        } else {
            item.path.clone()
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {:<12}", item.name), Style::default().fg(app.theme.text)),
            Span::styled(format!(" {:>9}", status), Style::default().fg(status_color)),
            Span::styled(format!(" {:>12}", item.version), Style::default().fg(app.theme.warning)),
            Span::styled(format!("  {}", path_display), Style::default().fg(app.theme.text_muted)),
        ]));
    }
    lines
}
fn render_gpu(app: &App) -> Vec<Line<'static>> {
    vec![Line::from(Span::styled(
        " GPU diagnostics not implemented",
        Style::default().fg(app.theme.text_muted),
    ))]
}
fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let controls = Line::from(vec![
        Span::styled(" [↑/↓] Navigate  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[R] Refresh  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[E] Export  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[Esc] Back  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[Q] Quit", Style::default().fg(app.theme.text_muted)),
    ]);
    let paragraph = Paragraph::new(controls)
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(app.theme.border)))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(area.height.saturating_sub(percent_y) / 2),
            Constraint::Length(percent_y),
            Constraint::Min(0),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(area.width.saturating_sub(percent_x) / 2),
            Constraint::Length(percent_x),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}
pub fn handle_event(app: &mut App, evt: event::Event) {
    let event = match evt {
        | event::Event::Key(key) => to_internal_event(key),
        | _ => None,
    };
    let Some(event) = event else {
        return;
    };
    match event {
        | Event::Quit => app.should_quit = true,
        | Event::Back => app.navigate_to(Screen::Dashboard),
        | Event::Up => {
            app.doctor.selected_category = app.doctor.selected_category.saturating_sub(1).min(CATEGORIES.len() - 1);
        }
        | Event::Down => {
            app.doctor.selected_category = app.doctor.selected_category.saturating_add(1).min(CATEGORIES.len() - 1);
        }
        | Event::Number(n) => {
            let idx = (n as usize).saturating_sub(1);
            if idx < CATEGORIES.len() {
                app.doctor.selected_category = idx;
            }
        }
        | Event::Refresh => {
            app.refresh_doctor();
        }
        | Event::Export => {
            if let Some(ref data) = app.doctor.data {
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let filename = format!("acorn_doctor_report_{timestamp}.json");
                match export::export_doctor_report(data, Some(std::path::Path::new(&filename))) {
                    | Ok(path) => {
                        app.doctor.export_message = Some(format!(" Report saved to {path} "));
                    }
                    | Err(e) => {
                        app.doctor.export_message = Some(format!(" Export failed: {e} "));
                    }
                }
            } else {
                app.doctor.export_message = Some(" No data to export ".into());
            }
        }
        | Event::Enter => {
            app.doctor.export_message = None;
        }
        | _ => {}
    }
}
