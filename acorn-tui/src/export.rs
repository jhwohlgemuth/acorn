use crate::app::DoctorData;
use chrono::Utc;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Export doctor diagnostic data as a JSON report
pub fn export_doctor_report(data: &DoctorData, path: Option<&Path>) -> Result<String, String> {
    let report = build_doctor_json(data);
    let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    match path {
        | Some(p) => {
            fs::write(p, &json).map_err(|e| format!("Failed to write report: {e}"))?;
            Ok(p.display().to_string())
        }
        | None => Ok(json),
    }
}
fn build_doctor_json(data: &DoctorData) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("report_type".into(), json!("doctor"));
    map.insert("timestamp".into(), json!(Utc::now().to_rfc3339()));
    if let Some(ref sys) = data.system {
        map.insert(
            "system".into(),
            json!({
                "name": sys.name,
                "kernel_version": sys.kernel,
                "os_version": sys.os_version,
                "host_name": sys.host_name,
                "cpu_architecture": sys.cpu_arch,
                "cpu_count": sys.cpu_count,
            }),
        );
    }
    if let Some(ref mem) = data.memory {
        map.insert(
            "memory".into(),
            json!({
                "total": mem.total,
                "available": mem.available,
                "used": mem.used,
                "swap": mem.swap,
            }),
        );
    }
    if let Some(ref net) = data.network {
        let interfaces: Vec<Value> = net
            .interfaces
            .iter()
            .map(|iface| {
                json!({
                    "ip_addresses": iface.ip_addresses,
                    "mac_address": iface.mac_address,
                    "mtu": iface.mtu,
                })
            })
            .collect();
        map.insert("network".into(), json!(interfaces));
    }
    if let Some(ref sw) = data.software {
        let tools: Vec<Value> = sw
            .items
            .iter()
            .map(|item| {
                json!({
                    "name": item.name,
                    "installed": item.installed,
                    "version": item.version,
                    "path": item.path,
                })
            })
            .collect();
        map.insert("software".into(), json!(tools));
    }
    Value::Object(map)
}
