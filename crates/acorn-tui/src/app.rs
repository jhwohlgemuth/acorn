use crate::screens;
use crate::theme::Theme;
use crate::{Screen, TuiOptions};
use acorn::analyzer::fix::Plan;
use acorn::analyzer::host::discovery::{Records, RemoteMatch, RemoteSearchResponse};
use acorn::analyzer::Check;
use acorn::io::api::huggingface;
use acorn_schema::discovery::RemoteOrganizationRole;
use color_eyre::eyre::Result;
use crossterm::event;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::BTreeSet;
use std::io::Stdout;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
/// Diagnostic records populated once on entering the doctor screen.
pub type DoctorData = Records;
/// Gather input mode.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GatherMode {
    /// Local document and PID discovery.
    #[default]
    Local,
    /// DOE CODE projects.
    OstiProjects,
    /// DOE CODE people.
    OstiPeople,
    /// DOE CODE organizations.
    OstiOrganizations,
}
/// Main application state
pub struct App {
    /// Whether the app should quit
    pub should_quit: bool,
    /// Current screen
    pub current_screen: Screen,
    /// Splash screen command composer state
    pub dashboard: State<DashboardData>,
    /// Doctor screen state
    pub doctor: DoctorState,
    /// Check screen state
    #[allow(dead_code)]
    pub check: CheckState,
    /// GGUF picker state
    pub gguf_picker: Option<State<GgufPickerData>>,
    /// Gather screen state
    pub gather: GatherState,
    /// Pending remote gather response.
    pub(crate) gather_remote: Option<Receiver<Result<RemoteSearchResponse, String>>>,
    /// Theme picker state
    pub theme_picker: State<ThemePickerData>,
    /// Current theme
    pub theme: Theme,
    /// Network and database settings inherited from the caller
    pub options: TuiOptions,
    /// When the theme was last changed (for debounce)
    theme_last_changed: Instant,
}
/// A GGUF repository presented by the interactive picker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// Hugging Face repository identifier
    pub id: String,
    /// Downloads during the repository's reported period
    pub downloads: u64,
    /// Repository like count
    pub likes: Option<u64>,
    /// Quantization formats detected in repository filenames
    pub quantizations: Vec<String>,
}
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CheckState {
    /// Check items (placeholder)
    pub items: Vec<String>,
    /// Selected index
    pub selected_index: usize,
    /// Whether loaded
    pub loaded: bool,
}
/// Data for the splash screen command composer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DashboardData {
    /// Text currently entered in the composer
    pub input: String,
    /// Directory in which the TUI started
    pub working_directory: String,
    /// Time at which the TUI started
    pub started_at: Instant,
}
#[derive(Clone, Debug)]
pub struct DoctorState {
    /// Cached diagnostic data
    pub data: Option<DoctorData>,
    /// Currently selected finding index.
    pub selected_record: usize,
    /// Finding indexes selected for trusted planning.
    pub selected: BTreeSet<usize>,
    /// Trusted plan awaiting confirmation.
    pub plan: Option<Plan>,
    /// Whether the plan confirmation prompt is active.
    pub confirming: bool,
    /// Whether diagnostics have been loaded
    pub loaded: bool,
    /// Export status message to display
    pub export_message: Option<String>,
}
/// Persistent identifier displayed by the gather screen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatherDiscovery {
    /// Normalized identifier value
    pub identifier: String,
    /// Identifier kind
    pub identifier_type: String,
    /// Source from which the identifier was discovered
    pub source: String,
    /// Detected source format
    pub source_format: String,
}
/// Interactive gather input and output state.
#[derive(Clone, Debug)]
pub struct GatherState {
    /// Checks produced while loading and persisting inputs
    pub checks: Vec<Check>,
    /// Discovered persistent identifiers
    pub discoveries: Vec<GatherDiscovery>,
    /// Current input value
    pub input: String,
    /// Number of processed sources
    pub input_count: usize,
    /// Selected discovery
    pub selected_index: usize,
    /// Active local or remote gather mode.
    pub mode: GatherMode,
    /// Remote provider matches.
    pub remote_matches: Vec<RemoteMatch>,
    /// Total matching upstream projects.
    pub remote_total: usize,
    /// Whether another remote page is available.
    pub remote_has_more: bool,
    /// Current remote project offset.
    pub remote_offset: usize,
    /// Whether a remote request is running.
    pub remote_loading: bool,
    /// Remote request error.
    pub remote_error: Option<String>,
    /// Treat the input as an organization filter.
    pub organization_filter: bool,
    /// Organization relationship filter.
    pub organization_role: RemoteOrganizationRole,
}
/// Data for selecting a GGUF repository.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GgufPickerData {
    /// Candidates sorted by downloads descending
    pub candidates: Vec<Candidate>,
    /// Base model used for discovery
    pub base_model: String,
    /// Selected repository ID
    pub result: Option<String>,
}
/// Selection state shared by interactive screens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State<T> {
    /// Screen-specific data
    pub data: T,
    /// Currently selected item index
    pub selected: usize,
}
/// Data for selecting a terminal theme.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThemePickerData {
    /// Available theme names
    pub names: Vec<&'static str>,
}
impl App {
    /// Create a new App with the given initial screen
    #[cfg(test)]
    pub fn new(initial_screen: Screen) -> Self {
        Self::with_options(initial_screen, TuiOptions::default())
    }
    /// Create application state with caller-provided runtime settings.
    pub fn with_options(initial_screen: Screen, options: TuiOptions) -> Self {
        Self {
            should_quit: false,
            current_screen: initial_screen,
            dashboard: State::new(DashboardData::new()),
            theme: Theme::from_env(),
            doctor: DoctorState::new(),
            check: CheckState::new(),
            gguf_picker: None,
            gather: GatherState::new(),
            gather_remote: None,
            theme_picker: State::new(ThemePickerData {
                names: Theme::NAMES.to_vec(),
            }),
            options,
            theme_last_changed: Instant::now(),
        }
    }
    /// Run the TUI event loop
    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        loop {
            screens::gather::poll_remote(self);
            terminal.draw(|f| self.render(f))?;
            if self.should_quit {
                break;
            }
            if event::poll(Duration::from_millis(100))? {
                let evt = event::read()?;
                self.handle_event(evt);
            }
        }
        Ok(())
    }
    fn render(&mut self, f: &mut ratatui::Frame) {
        match self.current_screen {
            | Screen::Dashboard => screens::dashboard::render(f, self),
            | Screen::Doctor => screens::doctor::render(f, self),
            | Screen::Check => screens::check::render(f, self),
            | Screen::GgufPicker => screens::gguf_picker::render(f, self),
            | Screen::Gather => screens::gather::render(f, self),
            | Screen::ThemePicker => screens::theme_picker::render(f, self),
        }
    }
    fn handle_event(&mut self, evt: event::Event) {
        if self.current_screen != Screen::Dashboard && self.current_screen != Screen::Gather {
            if let event::Event::Key(key) = &evt {
                if key.kind == event::KeyEventKind::Press && (key.code == event::KeyCode::Char('t') || key.code == event::KeyCode::Char('T')) {
                    self.cycle_theme();
                    return;
                }
            }
        }
        match self.current_screen {
            | Screen::Dashboard => screens::dashboard::handle_event(self, evt),
            | Screen::Doctor => screens::doctor::handle_event(self, evt),
            | Screen::Check => screens::check::handle_event(self, evt),
            | Screen::GgufPicker => screens::gguf_picker::handle_event(self, evt),
            | Screen::Gather => screens::gather::handle_event(self, evt),
            | Screen::ThemePicker => screens::theme_picker::handle_event(self, evt),
        }
    }
    /// Set the state used by the GGUF picker screen.
    pub fn set_gguf_picker(&mut self, state: State<GgufPickerData>) {
        self.gguf_picker = Some(state);
    }
    /// Take the repository ID selected by the GGUF picker.
    pub fn take_gguf_picker_result(&mut self) -> Option<String> {
        self.gguf_picker.as_mut().and_then(|state| state.data.result.take())
    }
    /// Cycle between available themes
    pub fn cycle_theme(&mut self) {
        const DEBOUNCE_MS: u64 = 200;
        if self.theme_last_changed.elapsed().as_millis() < DEBOUNCE_MS as u128 {
            return;
        }
        self.theme_last_changed = Instant::now();
        let current_index = Theme::NAMES.iter().position(|name| *name == self.theme.name).unwrap_or(0);
        let next_index = (current_index + 1) % Theme::NAMES.len();
        if let Some(theme) = Theme::named(Theme::NAMES[next_index]) {
            self.theme = theme;
        }
    }
    /// Select a theme by name.
    pub fn set_theme(&mut self, name: &str) {
        if let Some(theme) = Theme::named(name) {
            self.theme = theme;
        }
    }
    /// Navigate to a different screen
    pub fn navigate_to(&mut self, screen: Screen) {
        self.current_screen = screen;
    }
    /// Load diagnostic data from acorn-lib
    pub fn load_doctor_data(&mut self) {
        if self.doctor.loaded {
            return;
        }
        self.doctor.data = Some(Records::diagnose_host());
        self.doctor.loaded = true;
    }
    /// Refresh diagnostic data
    pub fn refresh_doctor(&mut self) {
        self.doctor.loaded = false;
        self.doctor.data = None;
        self.doctor.plan = None;
        self.doctor.confirming = false;
        self.doctor.selected.clear();
        self.load_doctor_data();
    }
}
impl From<huggingface::Candidate> for Candidate {
    fn from(candidate: huggingface::Candidate) -> Self {
        Self {
            id: candidate.id,
            downloads: candidate.downloads,
            likes: candidate.likes,
            quantizations: candidate.quantizations,
        }
    }
}
impl CheckState {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            loaded: false,
        }
    }
}
impl DashboardData {
    fn new() -> Self {
        Self {
            input: String::new(),
            working_directory: std::env::current_dir()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|_| String::from("unknown")),
            started_at: Instant::now(),
        }
    }
}
impl DoctorState {
    fn new() -> Self {
        Self {
            data: None,
            selected_record: 0,
            selected: BTreeSet::new(),
            plan: None,
            confirming: false,
            loaded: false,
            export_message: None,
        }
    }
}
impl GatherState {
    pub(crate) fn new() -> Self {
        Self {
            checks: Vec::new(),
            discoveries: Vec::new(),
            input: String::new(),
            input_count: 0,
            selected_index: 0,
            mode: GatherMode::Local,
            remote_matches: Vec::new(),
            remote_total: 0,
            remote_has_more: false,
            remote_offset: 0,
            remote_loading: false,
            remote_error: None,
            organization_filter: false,
            organization_role: RemoteOrganizationRole::Any,
        }
    }
}
impl<T> State<T> {
    /// Create screen state with the first item selected.
    pub fn new(data: T) -> Self {
        Self { data, selected: 0 }
    }
}
