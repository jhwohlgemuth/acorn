//! # Doctor module
//!
//! This module gathers and displays information about the user's system and repairs any issues that may prevent ACORN from running easily.
//!
//! Toward this end, `acorn doctor` will
//! * gather information about the system,
//! * display information about the system,
//! * and semi-automatically repair issues with the system.
//!
#[cfg(feature = "std")]
use crate::io::FromCommand;
use crate::schema::validate::is_ip6;
use crate::util::{print_values_as_table, to_string, Label, SemanticVersion};
use human_units::FormatSize;
use sysinfo::{Networks, System};
#[cfg(feature = "std")]
use which::which;

/// Trait for printing tables
pub trait TableFormatPrint {
    /// Initializes the struct
    fn init() -> Self;
    /// Prints the table
    fn print(self);
}
/// Information about installed software
#[derive(Debug, Clone)]
pub struct InstalledSoftwareData {
    /// Software version
    pub version: Option<String>,
    /// Path to software executable
    pub path: Option<String>,
}
/// Information about system memory
pub struct MemoryInformation {
    /// Total memory
    pub total: String,
    /// Available memory
    pub available: String,
    /// Used memory
    pub used: String,
    /// Swap memory
    pub swap: String,
}
/// Network details
#[derive(Debug, Clone)]
pub struct Network {
    /// IP Address
    pub ip_address: Vec<String>,
    /// MAC Address
    pub mac_address: String,
    /// Maximum transmission unit (MTU)
    pub mtu: String,
}
/// Information about system network
#[derive(Debug, Clone)]
pub struct NetworkInformation {
    /// List of networks
    pub networks: Vec<Network>,
}
/// Information about microprocessor
#[derive(Debug, Clone)]
pub struct Processor {
    /// CPU frequency (GHz)
    pub frequency: u64,
    /// CPU brand (e.g. AMD, Intel, etc...)
    pub brand: String,
    /// CPU vendor (e.g. Dell, Lenovo, etc...)
    pub vendor: String,
}
/// Information about system
pub struct SystemInformation {
    /// System name
    pub name: String,
    /// Kernel version
    pub kernel_version: String,
    /// Operating system (OS) version
    pub os_version: String,
    /// Host name
    pub host_name: String,
    /// CPU architecture
    pub cpu_arch: String,
    /// CPU count (number of cores)
    pub cpu_count: String,
}
/// Details of specific installed software applicable to ACORN
pub struct SystemSoftwareInformation {
    /// ### ACORN
    ///
    /// The subject of this documentation
    ///
    /// <div class="warning">See these <a href="https://code.ornl.gov/research-enablement/acorn#installation">installation instructions</a> for information on how to install the ACORN CLI application</div>
    pub acorn: InstalledSoftwareData,
    /// ### Git
    ///
    /// Version control system
    ///
    /// See <https://git-scm.com/>
    pub git: InstalledSoftwareData,
    /// ### Node.js
    ///
    /// Scripting language runtime
    ///
    /// See <https://nodejs.org/en>
    pub node: InstalledSoftwareData,
    /// ### npm
    ///
    /// [`Node.js`] package manager
    ///
    /// See <https://www.npmjs.com/>
    ///
    /// [`Node.js`]: ./struct.SystemSoftwareInformation.html#structfield.node
    pub npm: InstalledSoftwareData,
    /// ### npx
    ///
    /// Run a command from a local or remote npm package
    ///
    /// Bundled with [`npm`]
    ///
    /// [`npm`]: ./struct.SystemSoftwareInformation.html#structfield.npm
    pub npx: InstalledSoftwareData,
    /// ### Pandoc
    ///
    /// File conversion tool
    ///
    /// See <https://pandoc.org/>
    pub pandoc: InstalledSoftwareData,
    /// ### Vale
    ///
    /// Prose linter
    ///
    /// See <https://vale.sh/>
    pub vale: InstalledSoftwareData,
}
impl InstalledSoftwareData {
    fn from_command(name: &str) -> InstalledSoftwareData {
        InstalledSoftwareData {
            version: match SemanticVersion::from_command(name) {
                | Some(version) => Some(version.to_string()),
                | None => None,
            },
            path: match which(name) {
                | Ok(path) => Some(path.display().to_string()),
                | Err(_) => None,
            },
        }
    }
    fn as_row(&self, title: &str) -> Vec<String> {
        to_string(vec![
            title,
            &self.clone().is_installed(),
            &self.clone().version.unwrap_or_else(|| "---".to_string()),
            &self.clone().path.unwrap_or_else(|| "---".to_string()),
        ])
    }
    fn is_installed(&self) -> String {
        if self.version.is_some() {
            Label::CHECKMARK.to_string()
        } else {
            Label::CAUTION.to_string()
        }
    }
}
impl TableFormatPrint for MemoryInformation {
    fn init() -> MemoryInformation {
        let mut sys = System::new_all();
        sys.refresh_all();
        MemoryInformation {
            total: sys.total_memory().format_size().to_string(),
            available: sys.available_memory().format_size().to_string(),
            used: sys.used_memory().format_size().to_string(),
            swap: format!("{} of {}", sys.used_swap().format_size(), sys.total_swap().format_size()),
        }
    }
    fn print(self) {
        let MemoryInformation {
            total,
            available,
            used,
            swap,
        } = self;
        let headers = vec!["Attribute", "Value"];
        let rows = vec![
            to_string(vec!["Total", &total]),
            to_string(vec!["Available", &available]),
            to_string(vec!["Used", &used]),
            to_string(vec!["Swap", &swap]),
        ];
        print_values_as_table(headers, rows, Some("Memory"));
    }
}
impl TableFormatPrint for NetworkInformation {
    fn init() -> NetworkInformation {
        let networks = Networks::new_with_refreshed_list()
            .into_iter()
            .map(|(_, network)| Network {
                ip_address: parse_network_addresses(network),
                mac_address: network.mac_address().to_string(),
                mtu: network.mtu().to_string(),
            })
            .collect();
        NetworkInformation { networks }
    }
    fn print(self) {
        let headers = vec!["Network", "MAC Address", "MTU"];
        let rows = self
            .networks
            .iter()
            .filter_map(|network| {
                if network.ip_address.is_empty() {
                    None
                } else {
                    let ip = network.ip_address.join("\n");
                    let mac = network.clone().mac_address;
                    let mtu = network.clone().mtu;
                    Some(vec![ip, mac, mtu])
                }
            })
            .collect::<Vec<_>>();
        print_values_as_table(headers, rows, Some("Network"));
    }
}
impl TableFormatPrint for SystemInformation {
    fn init() -> SystemInformation {
        let mut sys = System::new_all();
        sys.refresh_all();
        SystemInformation {
            name: System::name().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            host_name: System::host_name().unwrap_or_default(),
            cpu_arch: System::cpu_arch(),
            cpu_count: sys.cpus().len().to_string(),
        }
    }
    fn print(self) {
        let SystemInformation {
            name,
            kernel_version,
            os_version,
            host_name,
            cpu_arch,
            cpu_count,
        } = self;
        let headers = vec!["Attribute", "Value"];
        let rows = vec![
            to_string(vec!["Name", &name]),
            to_string(vec!["Kernel Version", &kernel_version]),
            to_string(vec!["OS Version", &os_version]),
            to_string(vec!["Host Name", &host_name]),
            to_string(vec!["CPU Architecture", &cpu_arch]),
            to_string(vec!["CPU Count", &cpu_count]),
        ];
        print_values_as_table(headers, rows, Some("System"));
    }
}
impl TableFormatPrint for SystemSoftwareInformation {
    fn init() -> SystemSoftwareInformation {
        SystemSoftwareInformation {
            acorn: InstalledSoftwareData::from_command("acorn"),
            git: InstalledSoftwareData::from_command("git"),
            node: InstalledSoftwareData::from_command("node"),
            npm: InstalledSoftwareData::from_command("npm"),
            npx: InstalledSoftwareData::from_command("npx"),
            pandoc: InstalledSoftwareData::from_command("pandoc"),
            vale: InstalledSoftwareData::from_command("vale"),
        }
    }
    fn print(self) {
        let SystemSoftwareInformation {
            acorn,
            git,
            node,
            npm,
            npx,
            pandoc,
            vale,
        } = self;
        let headers = vec!["Name", "Installed", "Version", "Location"];
        let rows = vec![
            acorn.as_row("Acorn"),
            git.as_row("Git"),
            node.as_row("Node.js"),
            npm.as_row("npm"),
            npx.as_row("npx"),
            pandoc.as_row("Pandoc"),
            vale.as_row("Vale"),
        ];
        print_values_as_table(headers, rows, Some("Software"));
    }
}
fn parse_network_addresses(data: &sysinfo::NetworkData) -> Vec<String> {
    data.ip_networks()
        .iter()
        .map(|ip| ip.addr.to_string())
        .filter(|x| is_ip6(x).is_err())
        .collect::<Vec<String>>()
}
