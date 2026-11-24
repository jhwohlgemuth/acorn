use strum::EnumString;

#[derive(Clone, Copy, Debug, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum IconName {
    Alert,
    Book,
    Chart,
    Check,
    Code,
    Compass,
    Copy,
    Database,
    Diff,
    Download,
    Expand,
    External,
    Eye,
    File,
    #[strum(serialize = "filecheck")]
    FileCheck,
    Fold,
    FolderOpen,
    FontBig,
    FontSmall,
    Globe,
    Home,
    Info,
    Link,
    List,
    Maximize,
    Minimize,
    Panel,
    PanelOpen,
    Pause,
    Play,
    Portfolio,
    Search,
    Settings,
    Shield,
    Shuffle,
    SortAsc,
    SortDesc,
    SortTitle,
    Upload,
    Users,
    Wrap,
    X,
    Unknown,
}
impl IconName {
    pub(crate) fn legacy_artwork(self) -> &'static str {
        match self {
            | Self::Alert => r#"<path d="m10.3 4-8 14a2 2 0 0 0 1.7 3h16a2 2 0 0 0 1.7-3l-8-14a2 2 0 0 0-3.4 0ZM12 9v4m0 4h.01"/>"#,
            | Self::Book => {
                r#"<path class="s" d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20V4H6.5A2.5 2.5 0 0 0 4 6.5Z" />
      <path class="s" d="M4 6.5v13M9 10l2 2 4-4" />"#
            }
            | Self::Chart => r#"<path d="M4 3v17h17M7 16l4-3 4-5 5-4" fill="none" stroke="currentColor" stroke-width="1.6" />"#,
            | Self::Check => r#"<circle cx="12" cy="12" r="9"/><path d="m8 12 3 3 5-6"/>"#,
            | Self::Code => r#"<path d="m8 7-5 5 5 5m8-10 5 5-5 5m-3-13-2 16"/>"#,
            | Self::Compass => {
                r#"<circle class="s" cx="12" cy="12" r="9" />
      <path class="s" d="m16 8-2.5 5.5L8 16l2.5-5.5Z" />"#
            }
            | Self::Copy => {
                r#"<rect class="s" x="8" y="8" width="12" height="13" rx="2"/><path class="s" d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"/>"#
            }
            | Self::Database => {
                r#"<ellipse class="s" cx="12" cy="5" rx="8" ry="3"/><path class="s" d="M4 5v14c0 1.7 3.6 3 8 3s8-1.3 8-3V5M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3"/>"#
            }
            | Self::Diff => r#"<path class="s" d="M18 3v18M6 8V3M3 6l3-3 3 3M3 18l3 3 3-3M6 21v-5M15 7h6M15 17h6" />"#,
            | Self::Download => {
                r#"<path d="M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />"#
            }
            | Self::Expand => r#"<path d="m8 6 4-4 4 4M12 2v7m-4 9 4 4 4-4m-4 4v-7M4 12h16"/>"#,
            | Self::External => r#"<path class="s" d="M14 3h7v7M21 3l-9 9M20 14v6a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h6"/>"#,
            | Self::Eye => {
                r#"<path class="s" d="M2 12c2.5-4 6-6 10-6s7.5 2 10 6c-2.5 4-6 6-10 6S4.5 16 2 12Z"/><circle class="s" cx="12" cy="12" r="2.5"/>"#
            }
            | Self::File => r#"<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6M8 13h8M8 17h5"/>"#,
            | Self::FileCheck => {
                r#"<path class="s" d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z" />
      <path class="s" d="M14 2v6h6M9 15l2 2 4-4" />"#
            }
            | Self::Fold => r#"<path d="m8 3 4 4 4-4m-4-1v5m-4 14 4-4 4 4m-4 1v-5M4 12h16"/>"#,
            | Self::FolderOpen => {
                r#"<path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"/>"#
            }
            | Self::FontBig => r#"<path class="s" d="M2 19 8 5l6 14M4 14h8M18 18V7m-3 3 3-3 3 3"/>"#,
            | Self::FontSmall => r#"<path class="s" d="M2 19 8 5l6 14M4 14h8M18 7v11m-3-3 3 3 3-3"/>"#,
            | Self::Globe => {
                r#"<circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="1.5" />
      <ellipse cx="12" cy="12" rx="4" ry="9" fill="none" stroke="currentColor" stroke-width="1.5" />
      <path d="M3 12h18" stroke="currentColor" />"#
            }
            | Self::Home => r#"<path class="s" d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1Z" />"#,
            | Self::Info => {
                r#"<circle class="s" cx="12" cy="12" r="9" />
      <path class="s" d="M12 11v5M12 8h.01" />"#
            }
            | Self::Link => {
                r#"<path d="m10 13 4-4m-6 6-2 2a3.5 3.5 0 0 1-5-5l4-4a3.5 3.5 0 0 1 5 0m4 1 2-2a3.5 3.5 0 0 1 5 5l-4 4a3.5 3.5 0 0 1-5 0"/>"#
            }
            | Self::List => r#"<path class="s" d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" />"#,
            | Self::Maximize => r#"<path class="s" d="M8 3H3v5m13-5h5v5M3 16v5h5m13-5v5h-5" />"#,
            | Self::Minimize => r#"<path class="s" d="M3 8h5V3m13 5h-5V3M3 16h5v5m13-5h-5v5" />"#,
            | Self::Panel => {
                r#"<rect class="s" x="3" y="4" width="18" height="16" rx="2" />
      <path class="s" d="M9 4v16M14 8l-3 4 3 4" />"#
            }
            | Self::PanelOpen => r#"<rect class="s" x="3" y="4" width="18" height="16" rx="2" /><path class="s" d="M9 4v16m4-12 3 4-3 4" />"#,
            | Self::Pause => {
                r#"<rect class="s" x="6" y="4" width="4" height="16" rx="1"/><rect class="s" x="14" y="4" width="4" height="16" rx="1"/>"#
            }
            | Self::Play => r#"<path class="s" d="m8 4 12 8-12 8Z" />"#,
            | Self::Portfolio => {
                r#"<rect class="s" x="3" y="3" width="7" height="7" rx="1" />
      <rect class="s" x="14" y="3" width="7" height="7" rx="1" />
      <rect class="s" x="3" y="14" width="7" height="7" rx="1" />
      <rect class="s" x="14" y="14" width="7" height="7" rx="1" />"#
            }
            | Self::Search => {
                r#"<circle class="stroke" cx="11" cy="11" r="7" />
      <path class="stroke" d="m20 20-4-4" />"#
            }
            | Self::Settings => {
                r#"<circle class="s" cx="12" cy="12" r="3" />
      <path class="s" d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-1.6v-.2h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1Z" />"#
            }
            | Self::Shield => {
                r#"<path class="s" d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10Z" />
      <path class="s" d="m9 12 2 2 4-4" />"#
            }
            | Self::Shuffle => r#"<path class="s" d="M16 3h5v5M4 20 21 3M21 16v5h-5M15 15l6 6M4 4l5 5" />"#,
            | Self::SortAsc => r#"<path class="s" d="M7 20V4m-4 4 4-4 4 4M15 7h6m-6 5h5m-5 5h4"/>"#,
            | Self::SortDesc => r#"<path class="s" d="M7 4v16m-4-4 4 4 4-4M15 7h6m-6 5h5m-5 5h4"/>"#,
            | Self::SortTitle => r#"<path class="s" d="m3 20 5-16 5 16M5 14h6M16 6h5m-5 6h5m-5 6h5"/>"#,
            | Self::Upload => r#"<path d="M12 3v12"/><path d="m17 8-5-5-5 5"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>"#,
            | Self::Users => {
                r#"<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2m20 0v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/><circle cx="9" cy="7" r="4"/>"#
            }
            | Self::Wrap => r#"<path class="s" d="M3 6h18M3 10h12a4 4 0 0 1 0 8H9m3-3-3 3 3 3M3 14h5M3 18h3"/>"#,
            | Self::X => r#"<path class="s" d="M5 5l14 14M19 5 5 19"/>"#,
            | Self::Unknown => "",
        }
    }
    pub(crate) fn uswds_symbol(self) -> Option<&'static str> {
        match self {
            | Self::Alert => Some("warning"),
            | Self::Book => Some("menu_book"),
            | Self::Chart => Some("insights"),
            | Self::Check => Some("check_circle"),
            | Self::Code => Some("code"),
            | Self::Compass => Some("near_me"),
            | Self::Copy => Some("content_copy"),
            | Self::Database => Some("storage"),
            | Self::Diff => Some("difference"),
            | Self::Download => Some("file_download"),
            | Self::Expand => Some("unfold_more"),
            | Self::External => Some("launch"),
            | Self::Eye => Some("visibility"),
            | Self::File => Some("file_present"),
            | Self::FileCheck => Some("verified"),
            | Self::Fold => Some("unfold_less"),
            | Self::FolderOpen => Some("folder_open"),
            | Self::FontBig | Self::FontSmall => Some("format_size"),
            | Self::Globe => Some("public"),
            | Self::Home => Some("home"),
            | Self::Info => Some("info"),
            | Self::Link => Some("link"),
            | Self::List => Some("list"),
            | Self::Maximize => Some("fullscreen"),
            | Self::Minimize => Some("fullscreen_exit"),
            | Self::Panel => Some("first_page"),
            | Self::PanelOpen => Some("last_page"),
            | Self::Pause => Some("pause"),
            | Self::Play => Some("play_arrow"),
            | Self::Portfolio => Some("grid_view"),
            | Self::Search => Some("search"),
            | Self::Settings => Some("settings"),
            | Self::Shield => Some("verified_user"),
            | Self::Shuffle => Some("shuffle"),
            | Self::SortAsc => Some("arrow_upward"),
            | Self::SortDesc => Some("arrow_downward"),
            | Self::SortTitle => Some("sort_arrow"),
            | Self::Upload => Some("file_upload"),
            | Self::Users => Some("groups"),
            | Self::Wrap => Some("wrap_text"),
            | Self::X => Some("close"),
            | Self::Unknown => None,
        }
    }
}
