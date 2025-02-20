use crate::schema::validate::is_ip6;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use human_units::FormatSize;
use owo_colors::OwoColorize;
use sysinfo::{Networks, System};

pub trait TableFormatPrint {
    fn init() -> Self;
    fn print(self);
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
        let rows = [
            vec!["Total", &total],
            vec!["Available", &available],
            vec!["Used", &used],
            vec!["Swap", &swap],
        ];
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Attribute", "Value"]);
        rows.into_iter().for_each(|row| {
            table.add_row(row);
        });
        println!("=> {} \n{table}", "Memory".green().bold());
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
        let rows = self
            .networks
            .into_iter()
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
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Network", "MAC Address", "MTU"]);
        rows.into_iter().for_each(|row| {
            table.add_row(row);
        });
        println!("=> {} \n{table}", "Network".green().bold());
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
        let rows = [
            vec!["Name", &name],
            vec!["Kernel Version", &kernel_version],
            vec!["OS Version", &os_version],
            vec!["Host Name", &host_name],
            vec!["CPU Architecture", &cpu_arch],
            vec!["CPU Count", &cpu_count],
        ];
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Attribute", "Value"]);
        rows.into_iter().for_each(|row| {
            table.add_row(row);
        });
        println!("=> {} \n{table}", "System".green().bold());
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
