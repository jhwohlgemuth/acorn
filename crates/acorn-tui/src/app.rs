use crate::screens;
use crate::theme::Theme;
use crate::{Screen, TuiOptions};
use acorn::analyzer::discovery::{RemoteMatch, RemoteOrganizationRole, RemoteSearchResponse};
use acorn::analyzer::Check;
use acorn::doctor::{MemoryInformation, NetworkInformation, SystemInformation, SystemSoftwareInformation, TableFormatPrint};
use acorn::io::api::huggingface;
use color_eyre::eyre::Result;
use crossterm::event;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

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
/// Selection state shared by interactive screens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State<T> {
    /// Screen-specific data
    pub data: T,
    /// Currently selected item index
    pub selected: usize,
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
#[allow(dead_code)]
pub struct CheckState {
    /// Check items (placeholder)
    pub items: Vec<String>,
    /// Selected index
    pub selected_index: usize,
    /// Whether loaded
    pub loaded: bool,
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
/// Data for selecting a terminal theme.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThemePickerData {
    /// Available theme names
    pub names: Vec<&'static str>,
}
/// Diagnostic data populated once on entering doctor screen
#[derive(Clone, Debug)]
pub struct DoctorData {
    /// System information
    pub system: Option<SystemData>,
    /// Memory information
    pub memory: Option<MemoryData>,
    /// Network information
    pub network: Option<NetworkData>,
    /// Software information
    pub software: Option<SoftwareData>,
}
#[derive(Clone, Debug)]
pub struct DoctorState {
    /// Cached diagnostic data
    pub data: Option<DoctorData>,
    /// Currently selected category index
    pub selected_category: usize,
    /// Whether diagnostics have been loaded
    pub loaded: bool,
    /// Export status message to display
    pub export_message: Option<String>,
}
#[derive(Clone, Debug)]
pub struct InterfaceData {
    /// IP addresses
    pub ip_addresses: Vec<String>,
    /// MAC address
    pub mac_address: String,
    /// MTU
    pub mtu: String,
}
#[derive(Clone, Debug)]
pub struct MemoryData {
    /// Total memory
    pub total: String,
    /// Available memory
    pub available: String,
    /// Used memory
    pub used: String,
    /// Swap memory
    pub swap: String,
}
#[derive(Clone, Debug)]
pub struct NetworkData {
    /// Network interfaces
    pub interfaces: Vec<InterfaceData>,
}
#[derive(Clone, Debug)]
pub struct SoftwareData {
    /// List of software items
    pub items: Vec<SoftwareItem>,
}
#[derive(Clone, Debug)]
pub struct SoftwareItem {
    /// Software name
    pub name: String,
    /// Whether installed
    pub installed: bool,
    /// Version string
    pub version: String,
    /// Path to executable
    pub path: String,
}
#[derive(Clone, Debug)]
pub struct SystemData {
    /// OS name
    pub name: String,
    /// Kernel version
    pub kernel: String,
    /// OS version
    pub os_version: String,
    /// Host name
    pub host_name: String,
    /// CPU architecture
    pub cpu_arch: String,
    /// CPU count
    pub cpu_count: String,
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
        let sys = SystemInformation::init();
        let mem = MemoryInformation::init();
        let net = NetworkInformation::init();
        let sw = SystemSoftwareInformation::init();
        let software_items = vec![
            SoftwareItem {
                name: "Acorn".into(),
                installed: sw.acorn.version.is_some(),
                version: sw.acorn.version.unwrap_or_else(|| "---".into()),
                path: sw.acorn.path.unwrap_or_else(|| "---".into()),
            },
            SoftwareItem {
                name: "Git".into(),
                installed: sw.git.version.is_some(),
                version: sw.git.version.unwrap_or_else(|| "---".into()),
                path: sw.git.path.unwrap_or_else(|| "---".into()),
            },
            SoftwareItem {
                name: "Node.js".into(),
                installed: sw.node.version.is_some(),
                version: sw.node.version.unwrap_or_else(|| "---".into()),
                path: sw.node.path.unwrap_or_else(|| "---".into()),
            },
            SoftwareItem {
                name: "npm".into(),
                installed: sw.npm.version.is_some(),
                version: sw.npm.version.unwrap_or_else(|| "---".into()),
                path: sw.npm.path.unwrap_or_else(|| "---".into()),
            },
            SoftwareItem {
                name: "npx".into(),
                installed: sw.npx.version.is_some(),
                version: sw.npx.version.unwrap_or_else(|| "---".into()),
                path: sw.npx.path.unwrap_or_else(|| "---".into()),
            },
            SoftwareItem {
                name: "Pandoc".into(),
                installed: sw.pandoc.version.is_some(),
                version: sw.pandoc.version.unwrap_or_else(|| "---".into()),
                path: sw.pandoc.path.unwrap_or_else(|| "---".into()),
            },
            SoftwareItem {
                name: "Vale".into(),
                installed: sw.vale.version.is_some(),
                version: sw.vale.version.unwrap_or_else(|| "---".into()),
                path: sw.vale.path.unwrap_or_else(|| "---".into()),
            },
        ];
        self.doctor.data = Some(DoctorData {
            system: Some(SystemData {
                name: sys.name,
                kernel: sys.kernel_version,
                os_version: sys.os_version,
                host_name: sys.host_name,
                cpu_arch: sys.cpu_arch,
                cpu_count: sys.cpu_count,
            }),
            memory: Some(MemoryData {
                total: mem.total,
                available: mem.available,
                used: mem.used,
                swap: mem.swap,
            }),
            network: Some(NetworkData {
                interfaces: net
                    .networks
                    .iter()
                    .filter(|n| !n.ip_address.is_empty())
                    .map(|n| InterfaceData {
                        ip_addresses: n.ip_address.clone(),
                        mac_address: n.mac_address.clone(),
                        mtu: n.mtu.clone(),
                    })
                    .collect(),
            }),
            software: Some(SoftwareData { items: software_items }),
        });
        self.doctor.loaded = true;
    }
    /// Refresh diagnostic data
    pub fn refresh_doctor(&mut self) {
        self.doctor.loaded = false;
        self.doctor.data = None;
        self.load_doctor_data();
    }
}
impl<T> State<T> {
    /// Create screen state with the first item selected.
    pub fn new(data: T) -> Self {
        Self { data, selected: 0 }
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
impl CheckState {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            loaded: false,
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
impl DoctorState {
    fn new() -> Self {
        Self {
            data: None,
            selected_category: 0,
            loaded: false,
            export_message: None,
        }
    }
}
