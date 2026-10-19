//! HTML rendering for logbook report pages.
use crate::template::{generate_font_faces, DataUri, Style, ORNL_COLOR_LOGO, ORNL_LEAF_LOGO, ORNL_WHITE_LOGO};
use acorn::io::ApiResult;
use acorn::util::constants::app::REPORT_FULL_TIMESTAMP_FORMAT;
use acorn_schema::research_activity::{LogbookCategory, LogbookEntry, LogbookState, LogbookUpdates};
use acorn_schema::ClassificationLevel;
use quick_xml::escape::escape;

trait DisplayIcon {
    fn icon(self) -> String;
}

pub(crate) trait LogbookEntryRender {
    fn render(&self, full_timestamp: bool) -> String;
}

trait LogbookStateHeading {
    fn heading(self) -> (&'static str, &'static str);
}

pub(crate) trait LogbookUpdatesHtml {
    fn report_period(&self, full_timestamp: bool) -> String;
    fn render_status_group(&self, state: LogbookState, full_timestamp: bool) -> Option<String>;
    fn project_body(&self, full_timestamp: bool) -> String;
    fn title_body(&self, full_timestamp: bool) -> ApiResult<String>;
    fn to_html(&self, full_timestamp: bool) -> ApiResult<String> {
        html_document(self.project_body(full_timestamp))
    }
    fn to_title_html(&self, full_timestamp: bool) -> ApiResult<String> {
        self.title_body(full_timestamp).and_then(html_document)
    }
}

impl DisplayIcon for LogbookCategory {
    fn icon(self) -> String {
        let drawing = match self {
            | LogbookCategory::Administration => {
                "<rect x='3' y='5' width='18' height='16' rx='1'/><path d='M8 5V3h8v2M7 10h2m6 0h2M7 14h2m6 0h2M11 21v-4h2v4'/>"
            }
            | LogbookCategory::Collaboration => {
                "<circle cx='8' cy='8' r='3'/><circle cx='16' cy='8' r='3'/><path d='M2 19c0-3 2-5 6-5s6 2 6 5m-2-4c1-1 2-1 4-1 4 0 6 2 6 5'/>"
            }
            | LogbookCategory::Data => "<ellipse cx='12' cy='5' rx='8' ry='3'/><path d='M4 5v14c0 2 4 3 8 3s8-1 8-3V5M4 12c0 2 4 3 8 3s8-1 8-3'/>",
            | LogbookCategory::Engagement => "<path d='M3 10h4l11-5v14L7 14H3zM7 14l2 7h4l-2-6M20 9a4 4 0 0 1 0 6'/>",
            | LogbookCategory::Funding => "<circle cx='12' cy='12' r='9'/><path d='M15 8h-4a2 2 0 0 0 0 4h2a2 2 0 0 1 0 4H9M12 5v3m0 8v3'/>",
            | LogbookCategory::Infrastructure => "<rect x='3' y='3' width='18' height='18' rx='1'/><path d='M3 9h18M3 15h18M9 9v6M15 15v6'/>",
            | LogbookCategory::Method => "<path d='M5 4h14v17H5zM8 9l1 1 2-2m2 1h4M8 15l1 1 2-2m2 1h4'/>",
            | LogbookCategory::Other => "<path d='m12 3 2.7 6.3L21 12l-6.3 2.7L12 21l-2.7-6.3L3 12l6.3-2.7z'/>",
            | LogbookCategory::Personnel => "<circle cx='12' cy='8' r='4'/><path d='M4 21c0-5 3-8 8-8s8 3 8 8'/>",
            | LogbookCategory::Publication => "<path d='M12 20c-2-2-5-3-9-3V4c4 0 7 1 9 3 2-2 5-3 9-3v13c-4 0-7 1-9 3zm0-13v13'/>",
            | LogbookCategory::Research => "<path d='M9 3h6M10 3v7L5 19a2 2 0 0 0 2 3h10a2 2 0 0 0 2-3l-5-9V3M8 16h8'/>",
            | LogbookCategory::Software => "<path d='m8 7-5 5 5 5m8-10 5 5-5 5M14 4l-4 16'/>",
        };
        format!("<svg viewBox='0 0 24 24' aria-hidden='true'>{drawing}</svg>")
    }
}

impl LogbookEntryRender for LogbookEntry {
    fn render(&self, full_timestamp: bool) -> String {
        let summary = self
            .summary
            .split("\n\n")
            .map(str::trim)
            .filter(|paragraph| !paragraph.is_empty())
            .map(|paragraph| format!("<p>{}</p>", escape(paragraph)))
            .collect::<String>();
        let items = self
            .items
            .as_ref()
            .map(|items| {
                format!(
                    "<ul>{}</ul>",
                    items.iter().map(|item| format!("<li>{}</li>", escape(item.as_str()))).collect::<String>()
                )
            })
            .unwrap_or_default();
        let details = [
            self.event_type.map(|event| format!("Event: {event:?}")),
            Some(match self.discovery.as_ref() {
                | Some(discovery) => format!("Discovered via {:?}", discovery.provider),
                | None => "Manual entry".to_string(),
            }),
            self.classification
                .as_ref()
                .filter(|level| *level > &ClassificationLevel::Unclassified)
                .map(|level| format!("Classification: {level}")),
            self.archived.then(|| "Archived".to_string()),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");
        format!(
            "<article><h4>{} {:?}</h4><div class=\"entry-meta\"><time>{}</time><span class=\"details\">{}</span></div>{summary}{items}</article>",
            self.category.icon(),
            self.category,
            escape(display_timestamp(self.timestamp.as_str(), full_timestamp).as_str()),
            escape(details.as_str())
        )
    }
}

impl LogbookStateHeading for LogbookState {
    fn heading(self) -> (&'static str, &'static str) {
        match self {
            | LogbookState::Completed => ("completed", "Completed"),
            | LogbookState::InProgress => ("in-progress", "In progress"),
            | LogbookState::Upcoming => ("upcoming", "Upcoming"),
        }
    }
}

impl LogbookUpdatesHtml for LogbookUpdates {
    fn report_period(&self, full_timestamp: bool) -> String {
        self.through.as_ref().map_or_else(
            || format!("Since {}", escape(display_report_date(self.since.as_str(), full_timestamp).as_str())),
            |through| match self.since.as_str() {
                | "full-history" => format!("Through {}", escape(display_report_date(through.as_str(), full_timestamp).as_str())),
                | since => format!(
                    "{} - {}",
                    escape(display_report_date(since, full_timestamp).as_str()),
                    escape(display_report_date(through.as_str(), full_timestamp).as_str())
                ),
            },
        )
    }
    fn render_status_group(&self, state: LogbookState, full_timestamp: bool) -> Option<String> {
        let entries = self.entries.iter().filter(|entry| entry.state == state).collect::<Vec<_>>();
        (!entries.is_empty()).then(|| {
            let count = entries.len();
            let noun = if count == 1 { "entry" } else { "entries" };
            let articles = entries.into_iter().map(|entry| entry.render(full_timestamp)).collect::<String>();
            let (class, label) = state.heading();
            format!("<section class=\"status-group {class}\"><h3>{label}<span>{count} {noun}</span></h3>{articles}</section>")
        })
    }
    fn project_body(&self, full_timestamp: bool) -> String {
        let entries = [LogbookState::Completed, LogbookState::InProgress, LogbookState::Upcoming]
            .into_iter()
            .filter_map(|state| self.render_status_group(state, full_timestamp))
            .collect::<String>();
        let credits = [
            ("Principal Investigator", "Principal Investigators", &self.principal_investigators),
            ("Sponsor", "Sponsors", &self.sponsors),
        ]
        .into_iter()
        .filter(|(_, _, names)| !names.is_empty())
        .map(|(singular, plural, names)| {
            let label = if names.len() == 1 { singular } else { plural };
            let values = names.iter().map(|name| escape(name).to_string()).collect::<Vec<_>>().join("; ");
            format!("<div><dt>{label}</dt><dd>{values}</dd></div>")
        })
        .collect::<String>();
        let credits = if credits.is_empty() {
            String::new()
        } else {
            format!("<dl class=\"project-credits\">{credits}</dl>")
        };
        let period = self.report_period(full_timestamp);
        let subtitle = self
            .subtitle
            .as_deref()
            .filter(|subtitle| !subtitle.trim().is_empty())
            .map(|subtitle| format!("<p class=\"project-subtitle\">{}</p>", escape(subtitle)))
            .unwrap_or_default();
        let keywords = self
            .keywords
            .iter()
            .map(|keyword| keyword.trim())
            .filter(|keyword| !keyword.is_empty())
            .map(|keyword| format!("<li>{}</li>", escape(keyword)))
            .collect::<String>();
        let keywords = if keywords.is_empty() {
            String::new()
        } else {
            format!("<ul class=\"project-keywords\" aria-label=\"Project topics\">{keywords}</ul>")
        };
        format!(
            "<header><h1>{}</h1>{subtitle}{keywords}<div class=\"report-heading\"><h2>Project activity highlights</h2><p class=\"period\">{period}</p></div></header><main>{credits}{}</main>",
            escape(self.title.as_str()),
            match entries.is_empty() {
                | true => "<p><em>No updates in this period.</em></p>".to_string(),
                | false => entries,
            }
        )
    }
    fn title_body(&self, full_timestamp: bool) -> ApiResult<String> {
        let month = self
            .since
            .parse::<jiff::Timestamp>()
            .map(|timestamp| format!("{} Report", timestamp.strftime("%B %Y")))
            .unwrap_or_else(|_| "Activity Report".to_string());
        let entries = self.entries.iter().map(|entry| entry.render(full_timestamp)).collect::<String>();
        DataUri::from_asset(ORNL_WHITE_LOGO)
            .and_then(|logo| DataUri::from_asset(ORNL_LEAF_LOGO).map(|leaf| (logo, leaf)))
            .and_then(|(logo, leaf)| DataUri::from_asset("report/title-banner.png").map(|banner| (logo, leaf, banner)))
            .map(|(logo, leaf, banner)| format!(
                "<section class=\"title-page\"><div class=\"title-banner\" style=\"background-image:url('{banner}')\"><img class=\"title-logo\" src=\"{logo}\" alt=\"Oak Ridge National Laboratory\"><img class=\"title-leaf\" src=\"{leaf}\" alt=\"\"><div class=\"title-block\"><h1>Project Activity Highlights</h1><p class=\"report-month\">{month}</p><p class=\"period\">{}</p></div></div><main class=\"general-updates\"><h2>General updates</h2>{}</main></section>",
                self.report_period(full_timestamp),
                match entries.is_empty() {
                    | true => "<p><em>No general updates in this period.</em></p>".to_string(),
                    | false => entries,
                }
            ))
    }
}

pub(crate) fn combined_html(title: &LogbookUpdates, projects: &[LogbookUpdates], full_timestamp: bool) -> ApiResult<String> {
    title.title_body(full_timestamp).and_then(|title| {
        let projects = projects
            .iter()
            .map(|project| format!("<section class=\"project-page\">{}</section>", project.project_body(full_timestamp)))
            .collect::<String>();
        html_document(format!("<div class=\"combined-report\">{title}{projects}{}</div>", legend_body()))
    })
}

fn display_report_date(value: &str, full_timestamp: bool) -> String {
    if full_timestamp {
        display_timestamp(value, true)
    } else {
        value
            .parse::<jiff::Timestamp>()
            .map(|timestamp| timestamp.strftime("%d%b").to_string().to_uppercase())
            .unwrap_or_else(|_| value.to_string())
    }
}

pub(crate) fn display_timestamp(value: &str, full_timestamp: bool) -> String {
    value
        .parse::<jiff::Timestamp>()
        .map(|timestamp| {
            let format = if full_timestamp { REPORT_FULL_TIMESTAMP_FORMAT } else { "%B %-d" };
            timestamp.strftime(format).to_string()
        })
        .unwrap_or_else(|_| value.to_string())
}

fn html_document(body: String) -> ApiResult<String> {
    DataUri::from_asset(ORNL_COLOR_LOGO).and_then(|footer_logo| {
        generate_font_faces().and_then(|fonts| {
            Style::to_string("updates.css".to_string()).map(|style| {
                format!(
                    "<!doctype html><html><head><meta charset=\"utf-8\"><style>{fonts}{style}@page {{ @bottom-left {{ background-image: url('{footer_logo}'); }} }}</style></head><body>{body}</body></html>"
                )
            })
        })
    })
}

fn legend_body() -> String {
    let statuses = [
        (LogbookState::Completed, "Teal", "Work finished at the recorded date."),
        (LogbookState::InProgress, "Blue", "Work underway at the recorded date."),
        (LogbookState::Upcoming, "Amber", "Work expected at the recorded date."),
    ]
    .into_iter()
    .map(|(state, color, meaning)| {
        let (class, label) = state.heading();
        format!("<div class=\"legend-status {class}\"><h3>{label}</h3><p><strong>{color}.</strong> {meaning}</p></div>")
    })
    .collect::<String>();
    let categories = [
        (LogbookCategory::Administration, "Project governance, coordination, and records."),
        (LogbookCategory::Collaboration, "Partnerships and joint working groups."),
        (LogbookCategory::Data, "Data collection, curation, release, and stewardship."),
        (LogbookCategory::Engagement, "Communication with audiences, sponsors, or stakeholders."),
        (LogbookCategory::Funding, "Proposals, awards, and funding management."),
        (LogbookCategory::Infrastructure, "Facilities, equipment, and technical platforms."),
        (LogbookCategory::Method, "Methods, protocols, and research workflows."),
        (LogbookCategory::Other, "Activity outside the listed categories."),
        (LogbookCategory::Personnel, "Team membership, staffing, and training."),
        (LogbookCategory::Publication, "Papers, reports, and other published outputs."),
        (LogbookCategory::Research, "Experiments, analysis, and findings."),
        (LogbookCategory::Software, "Code, repositories, tools, and releases."),
    ]
    .into_iter()
    .map(|(category, meaning)| {
        format!(
            "<div class=\"legend-category\">{}<div><h3>{category:?}</h3><p>{meaning}</p></div></div>",
            category.icon()
        )
    })
    .collect::<String>();
    format!(
        "<section class=\"legend-page\"><header><h1>Reading this report</h1><p>Statuses and activity categories</p></header><main><section class=\"legend-section\"><h2>Status</h2><p class=\"legend-intro\">The heading color shows the state of work when an update was recorded.</p><div class=\"legend-statuses\">{statuses}</div></section><section class=\"legend-section\"><h2>Activity categories</h2><p class=\"legend-intro\">Category icons use the text color of their status heading; general updates use blue.</p><div class=\"legend-categories\">{categories}</div></section><p class=\"legend-note\">Status describes progress; category describes the kind of activity. Labels accompany every color and icon.</p></main></section>"
    )
}

pub(crate) fn legend_html() -> ApiResult<String> {
    html_document(legend_body())
}
