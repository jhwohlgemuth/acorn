use crate::schema::validate::is_ip6;
use crate::util::{print_values_as_table, to_string, Label, SemanticVersion};
use human_units::FormatSize;
use sysinfo::{Networks, System};
use which::which;

pub trait TableFormatPrint {
    fn init() -> Self;
    fn print(self);
}
#[derive(Debug, Clone)]
pub struct InstalledSoftwareData {
    pub version: Option<String>,
    pub path: Option<String>,
}
pub struct MemoryInformation {
    pub total: String,
    pub available: String,
    pub used: String,
    pub swap: String,
}
#[derive(Debug, Clone)]
pub struct Network {
    pub ip_address: Vec<String>,
    pub mac_address: String,
    pub mtu: String,
}
#[derive(Debug, Clone)]
pub struct NetworkInformation {
    pub networks: Vec<Network>,
}
// TODO: Add detailed CPU information printer
/// Information about microprocessor
#[derive(Debug, Clone)]
pub struct Processor {
    pub frequency: u64,
    pub vendor: String,
    pub brand: String,
}
pub struct SystemInformation {
    pub name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub host_name: String,
    pub cpu_arch: String,
    pub cpu_count: String,
}
pub struct SystemSoftwareInformation {
    pub acorn: InstalledSoftwareData,
    pub git: InstalledSoftwareData,
    pub node: InstalledSoftwareData,
    pub npm: InstalledSoftwareData,
    pub npx: InstalledSoftwareData,
    pub pandoc: InstalledSoftwareData,
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
        print_values_as_table("Memory", headers, rows);
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
        print_values_as_table("Network", headers, rows);
    }
}
impl TableFormatPrint for SystemInformation {
    fn init() -> SystemInformation {
        let mut sys = System::new_all();
        sys.refresh_all();
        SystemInformation {
            name: System::name().unwrap(),
            kernel_version: System::kernel_version().unwrap(),
            os_version: System::os_version().unwrap(),
            host_name: System::host_name().unwrap(),
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
        print_values_as_table("System", headers, rows);
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
        print_values_as_table("Software", headers, rows);
    }
}
pub fn print_system_information() {
    SystemInformation::init().print();
    NetworkInformation::init().print();
    MemoryInformation::init().print();
}
fn parse_network_addresses(data: &sysinfo::NetworkData) -> Vec<String> {
    data.ip_networks()
        .iter()
        .map(|ip| ip.addr.to_string())
        .filter(|x| is_ip6(x).is_err())
        .collect::<Vec<String>>()
}
