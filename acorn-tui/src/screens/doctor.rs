use crate::app::App;
use crate::event::{to_internal_event, Event};
use crate::export;
use crate::Screen;
use crossterm::event;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
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
    render_status_bar(f, chunks[1]);
}
fn render_category_list(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<Line> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_selected = i == app.doctor.selected_category;
            let prefix = if is_selected { ">" } else { " " };
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
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
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}
fn render_detail_pane(f: &mut Frame, area: Rect, app: &App) {
    let Some(ref data) = app.doctor.data else {
        let paragraph = Paragraph::new(" Loading diagnostics...")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    };
    let lines: Vec<Line> = match app.doctor.selected_category {
        | 0 => render_system(data),
        | 1 => render_memory(data),
        | 2 => render_network(data),
        | 3 => render_software(data),
        | 4 => render_gpu(),
        | _ => vec![Line::from(" Unknown category")],
    };
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", CATEGORIES[app.doctor.selected_category]))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Cyan)),
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
                    .border_style(Style::default().fg(Color::Green)),
            )
            .alignment(Alignment::Center);
        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);
    }
}
fn render_system(data: &crate::app::DoctorData) -> Vec<Line<'static>> {
    let Some(ref sys) = data.system else {
        return vec![Line::from(" No system data available")];
    };
    vec![
        Line::from(Span::styled(
            format!(" Name:             {}", sys.name),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!(" Kernel Version:   {}", sys.kernel),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!(" OS Version:       {}", sys.os_version),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!(" Host Name:        {}", sys.host_name),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!(" CPU Architecture: {}", sys.cpu_arch),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!(" CPU Count:        {}", sys.cpu_count),
            Style::default().fg(Color::White),
        )),
    ]
}
fn render_memory(data: &crate::app::DoctorData) -> Vec<Line<'static>> {
    let Some(ref mem) = data.memory else {
        return vec![Line::from(" No memory data available")];
    };
    vec![
        Line::from(Span::styled(format!(" Total:     {}", mem.total), Style::default().fg(Color::White))),
        Line::from(Span::styled(format!(" Available: {}", mem.available), Style::default().fg(Color::Green))),
        Line::from(Span::styled(format!(" Used:      {}", mem.used), Style::default().fg(Color::Yellow))),
        Line::from(Span::styled(format!(" Swap:      {}", mem.swap), Style::default().fg(Color::White))),
    ]
}
fn render_network(data: &crate::app::DoctorData) -> Vec<Line<'static>> {
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
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        for ip in &iface.ip_addresses {
            lines.push(Line::from(Span::styled(format!("   IP:  {ip}"), Style::default().fg(Color::White))));
        }
        lines.push(Line::from(Span::styled(
            format!("   MAC: {}", iface.mac_address),
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(Span::styled(
            format!("   MTU: {}", iface.mtu),
            Style::default().fg(Color::White),
        )));
    }
    lines
}
fn render_software(data: &crate::app::DoctorData) -> Vec<Line<'static>> {
    let Some(ref sw) = data.software else {
        return vec![Line::from(" No software data available")];
    };
    let mut lines = vec![Line::from(Span::styled(
        format!(" {:<12} {:>9} {:>12}  {}", "Name", "Installed", "Version", "Path"),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ))];
    lines.push(Line::from(Span::styled("-".repeat(60), Style::default().fg(Color::DarkGray))));
    for item in &sw.items {
        let status = if item.installed { "✓" } else { "✗" };
        let status_color = if item.installed { Color::Green } else { Color::Red };
        let path_display = if item.path.len() > 30 {
            format!("...{}", &item.path[item.path.len().saturating_sub(27)..])
        } else {
            item.path.clone()
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {:<12}", item.name), Style::default().fg(Color::White)),
            Span::styled(format!(" {:>9}", status), Style::default().fg(status_color)),
            Span::styled(format!(" {:>12}", item.version), Style::default().fg(Color::Yellow)),
            Span::styled(format!("  {}", path_display), Style::default().fg(Color::DarkGray)),
        ]));
    }
    lines
}
fn render_gpu() -> Vec<Line<'static>> {
    vec![Line::from(Span::styled(
        " GPU diagnostics not implemented",
        Style::default().fg(Color::DarkGray),
    ))]
}
fn render_status_bar(f: &mut Frame, area: Rect) {
    let controls = Line::from(vec![
        Span::styled(" [↑/↓] Navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[R] Refresh  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[E] Export  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc] Back  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Q] Quit", Style::default().fg(Color::DarkGray)),
    ]);
    let paragraph = Paragraph::new(controls)
        .block(Block::default().borders(Borders::TOP))
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
