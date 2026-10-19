use super::super::html::{display_timestamp, legend_html, LogbookEntryRender, LogbookUpdatesHtml};
use acorn_schema::research_activity::{LogbookEntry, LogbookUpdates};

#[test]
fn test_legend_page_covers_all_statuses_and_category_icons() {
    let html = legend_html().expect("legend HTML");
    assert_eq!(html.matches("class=\"legend-status ").count(), 3);
    assert_eq!(html.matches("class=\"legend-category\"").count(), 12);
    assert_eq!(html.matches("<svg").count(), 12);
    assert!(html.contains("Reading this report"));
    assert!(html.contains("Status describes progress; category describes the kind of activity."));
}
#[test]
fn test_pdf_timestamp_display() {
    let value = "2026-10-13T14:30:00Z";
    assert_eq!(display_timestamp(value, false), "October 13");
    assert_eq!(display_timestamp(value, true), "October 13, 2026 at 14:30 UTC");
}
#[test]
fn test_project_page_places_credits_before_status_groups() {
    let entry: LogbookEntry = serde_json::from_str(
            r#"{"identifier":"sample-entry","projectId":"sample-project","timestamp":"2026-10-13T14:30:00Z","category":"research","state":"completed","summary":"Completed sample work."}"#,
        )
        .expect("sample entry");
    let updates = LogbookUpdates {
        identifier: "sample-project".to_string(),
        title: "Sample Project".to_string(),
        subtitle: Some("Pilot subtitle".to_string()),
        keywords: vec!["Earth science".to_string(), "Data & sensing".to_string()],
        principal_investigators: vec!["Alex Morgan".to_string(), "Jordan & Lee".to_string()],
        sponsors: vec!["Sample Sponsor".to_string()],
        since: "2026-10-01T00:00:00Z".to_string(),
        through: None,
        entries: vec![entry],
    };
    let html = updates.to_html(false).expect("project HTML");
    let credits = html.find("<dl class=\"project-credits\">").expect("project credits");
    let status = html.find("status-group completed").expect("completed section");
    assert!(html.find("</header>").expect("banner") < credits && credits < status);
    assert!(html.contains("<dt>Principal Investigators</dt><dd>Alex Morgan; Jordan &amp; Lee</dd>"));
    assert!(html.contains("<dt>Sponsor</dt><dd>Sample Sponsor</dd>"));
    assert!(html.contains("<ul class=\"project-keywords\" aria-label=\"Project topics\"><li>Earth science</li><li>Data &amp; sensing</li></ul>"));
    let subtitle = html.find("class=\"project-subtitle\"").expect("subtitle");
    let keywords = html.find("<ul class=\"project-keywords\"").expect("project topics");
    let report_heading = html.find("<div class=\"report-heading\">").expect("report heading");
    assert!(subtitle < keywords && keywords < report_heading);
    assert!(html.contains("<div class=\"report-heading\"><h2>Project activity highlights</h2><p class=\"period\">Since 01OCT</p></div>"));
    let single_pi = LogbookUpdates {
        principal_investigators: vec!["Alex Morgan".to_string()],
        ..updates
    };
    assert!(single_pi
        .to_html(false)
        .expect("project HTML")
        .contains("<dt>Principal Investigator</dt>"));
}
#[test]
fn test_render_entry_places_date_and_event_on_one_line() {
    let entry: LogbookEntry = serde_json::from_str(
            r#"{"identifier":"sample-entry","projectId":"sample-project","timestamp":"2026-10-13T14:30:00Z","category":"engagement","eventType":"meeting","state":"completed","summary":"Met with collaborators."}"#,
        )
        .expect("sample entry");
    let html = entry.render(false);
    assert!(html.contains("<div class=\"entry-meta\"><time>October 13</time><span class=\"details\">Event: Meeting · Manual entry</span></div>"));
    let full_timestamp_html = entry.render(true);
    assert!(full_timestamp_html.contains("<time>October 13, 2026 at 14:30 UTC</time><span class=\"details\">Event: Meeting"));
}
#[test]
fn test_render_entry_preserves_summary_paragraphs() {
    let entry: LogbookEntry = serde_json::from_str(
            r#"{"identifier":"sample-entry","projectId":"sample-project","timestamp":"2026-10-13T14:30:00Z","category":"research","state":"completed","summary":"First paragraph.\n\nSecond paragraph with <markup>."}"#,
        )
        .expect("sample entry");
    let html = entry.render(false);
    assert!(html.contains("<p>First paragraph.</p><p>Second paragraph with &lt;markup&gt;.</p>"));
}
#[test]
fn test_report_period_omits_exclusive_label() {
    let updates = LogbookUpdates {
        identifier: "sample-project".to_string(),
        title: "Sample Project".to_string(),
        subtitle: None,
        keywords: Vec::new(),
        principal_investigators: Vec::new(),
        sponsors: Vec::new(),
        since: "2026-10-01T00:00:00Z".to_string(),
        through: Some("2026-11-01T00:00:00Z".to_string()),
        entries: Vec::new(),
    };
    assert_eq!(updates.report_period(false), "01OCT - 01NOV");
    assert_eq!(
        updates.report_period(true),
        "October 1, 2026 at 00:00 UTC - November 1, 2026 at 00:00 UTC"
    );
    assert_eq!(
        LogbookUpdates {
            since: "full-history".to_string(),
            ..updates
        }
        .report_period(false),
        "Through 01NOV"
    );
}
#[test]
fn test_title_page_contains_the_header_logo_only() {
    let updates = LogbookUpdates {
        identifier: "general-updates".to_string(),
        title: "General Updates".to_string(),
        subtitle: None,
        keywords: Vec::new(),
        principal_investigators: Vec::new(),
        sponsors: Vec::new(),
        since: "2026-10-01T00:00:00Z".to_string(),
        through: Some("2026-11-01T00:00:00Z".to_string()),
        entries: Vec::new(),
    };
    let title = updates.to_title_html(false).expect("title HTML");
    let project = updates.to_html(false).expect("project HTML");
    assert!(title.contains("alt=\"Oak Ridge National Laboratory\""));
    assert!(title.contains("class=\"title-leaf\""));
    assert!(title.contains("class=\"title-banner\" style=\"background-image:url('data:image/png;base64,"));
    assert!(title.contains("October 2026 Report"));
    assert!(title.contains("General updates"));
    assert!(!project.contains("alt=\"Oak Ridge National Laboratory\""));
    assert!(!project.contains("class=\"title-leaf\""));
    assert!(!project.contains("<ul class=\"project-keywords\""));
}
