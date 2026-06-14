use crate::screens;
use crate::theme::Theme;
use crate::Screen;
use acorn::doctor::{MemoryInformation, NetworkInformation, SystemInformation, SystemSoftwareInformation, TableFormatPrint};
use color_eyre::eyre::Result;
use crossterm::event;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::time::{Duration, Instant};

type ThemeBuilder = fn() -> Theme;

/// Main application state
pub struct App {
    /// Whether the app should quit
    pub should_quit: bool,
    /// Current screen
    pub current_screen: Screen,
    /// Doctor screen state
    pub doctor: DoctorState,
    /// Check screen state
    #[allow(dead_code)]
    pub check: CheckState,
    /// Current theme
    pub theme: Theme,
    /// When the theme was last changed (for debounce)
    theme_last_changed: Instant,
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
    pub fn new(initial_screen: Screen) -> Self {
        Self {
            should_quit: false,
            current_screen: initial_screen,
            theme: Theme::from_env(),
            doctor: DoctorState::new(),
            check: CheckState::new(),
            theme_last_changed: Instant::now(),
        }
    }
    /// Run the TUI event loop
    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        loop {
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
        }
    }
    fn handle_event(&mut self, evt: event::Event) {
        if let event::Event::Key(key) = &evt {
            if key.kind == event::KeyEventKind::Press && (key.code == event::KeyCode::Char('t') || key.code == event::KeyCode::Char('T')) {
                self.cycle_theme();
                return;
            }
        }
        match self.current_screen {
            | Screen::Dashboard => screens::dashboard::handle_event(self, evt),
            | Screen::Doctor => screens::doctor::handle_event(self, evt),
            | Screen::Check => screens::check::handle_event(self, evt),
        }
    }
    /// Cycle between available themes
    pub fn cycle_theme(&mut self) {
        const DEBOUNCE_MS: u64 = 200;
        if self.theme_last_changed.elapsed().as_millis() < DEBOUNCE_MS as u128 {
            return;
        }
        self.theme_last_changed = Instant::now();
        const THEMES: &[(ThemeBuilder, &str)] = &[(Theme::nord, "nord"), (Theme::one_dark, "one-dark")];
        let current_idx = THEMES.iter().position(|(_, name)| *name == self.theme.name).unwrap_or(0);
        let next_idx = (current_idx + 1) % THEMES.len();
        self.theme = THEMES[next_idx].0();
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
impl CheckState {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            loaded: false,
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
