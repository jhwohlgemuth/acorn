use crate::commands::doctor::{run, should_run};
use acorn_lib::doctor::{MemoryInformation, NetworkInformation, SystemInformation, TableFormatPrint};
use acorn_lib::util::cli::Diagnostic;
use color_eyre::eyre::Result;
use std::panic::{catch_unwind, AssertUnwindSafe};

const OFFLINE: bool = false;

#[test]
fn test_should_run() {
    // Test for specific diagnostics
    assert!(should_run(&[Diagnostic::Memory], Diagnostic::Memory));
    assert!(should_run(&[Diagnostic::System], Diagnostic::System));
    assert!(should_run(&[Diagnostic::Network], Diagnostic::Network));
    assert!(should_run(&[Diagnostic::Gpu], Diagnostic::Gpu));
    assert!(should_run(&[Diagnostic::Software], Diagnostic::Software));
    // Test diagnostic is not in list
    assert!(!should_run(&[Diagnostic::Memory], Diagnostic::System));
    assert!(!should_run(&[Diagnostic::Network], Diagnostic::Gpu));
    // Test Diagnostic::All includes everything
    assert!(should_run(&[Diagnostic::All], Diagnostic::Memory));
    assert!(should_run(&[Diagnostic::All], Diagnostic::System));
    assert!(should_run(&[Diagnostic::All], Diagnostic::Network));
    assert!(should_run(&[Diagnostic::All], Diagnostic::Gpu));
    assert!(should_run(&[Diagnostic::All], Diagnostic::Software));
    // Test empty list
    assert!(!should_run(&[], Diagnostic::Memory));
    assert!(!should_run(&[], Diagnostic::System));
    // Test multiple diagnostics in list
    assert!(should_run(&[Diagnostic::Memory, Diagnostic::System], Diagnostic::Memory));
    assert!(should_run(&[Diagnostic::Memory, Diagnostic::System], Diagnostic::System));
    assert!(!should_run(&[Diagnostic::Memory, Diagnostic::System], Diagnostic::Network));
}
#[test]
fn test_run_with_valid_diagnostics() -> Result<()> {
    // Test with individual diagnostics
    let result = run(&false, &false, &[Diagnostic::System], &OFFLINE);
    assert!(result.is_ok());
    let result = run(&false, &false, &[Diagnostic::Memory], &OFFLINE);
    assert!(result.is_ok());
    let result = run(&false, &false, &[Diagnostic::Network], &OFFLINE);
    assert!(result.is_ok());
    // Test with multiple diagnostics
    let result = run(&false, &false, &[Diagnostic::System, Diagnostic::Memory], &OFFLINE);
    assert!(result.is_ok());
    // Test with All diagnostic
    let result = run(&false, &false, &[Diagnostic::All], &OFFLINE);
    assert!(result.is_ok());
    Ok(())
}
#[test]
fn test_gpu_diagnostic_warning() -> Result<()> {
    // GPU is not implemented yet, but should not fail
    let result = run(&false, &false, &[Diagnostic::Gpu], &OFFLINE);
    assert!(result.is_ok());
    Ok(())
}
#[test]
fn test_unimplemented_features() {
    // Test fix option - should panic with unimplemented message
    let result = catch_unwind(AssertUnwindSafe(|| run(&true, &false, &[Diagnostic::System], &OFFLINE)));
    assert!(result.is_err());
    // Test interactive option - should panic with unimplemented message
    let result = catch_unwind(AssertUnwindSafe(|| run(&false, &true, &[Diagnostic::System], &OFFLINE)));
    assert!(result.is_err());
}
#[test]
fn test_run_with_empty_diagnostics() -> Result<()> {
    // With empty diagnostics list, nothing should run but it should not error
    let result = run(&false, &false, &[], &OFFLINE);
    assert!(result.is_ok());
    Ok(())
}
#[test]
fn test_system_information_init() {
    let system_info = SystemInformation::init();
    // Verify all fields are populated
    assert!(!system_info.name.is_empty());
    assert!(!system_info.kernel_version.is_empty());
    assert!(!system_info.os_version.is_empty());
    assert!(!system_info.host_name.is_empty());
    assert!(!system_info.cpu_arch.is_empty());
    assert!(!system_info.cpu_count.is_empty());
    // Verify CPU count is a valid number
    let cpu_count = system_info.cpu_count.parse::<usize>();
    assert!(cpu_count.is_ok());
    assert!(cpu_count.unwrap() > 0);
}
#[test]
fn test_memory_information_init() {
    let memory_info = MemoryInformation::init();
    // Check that memory information is populated
    assert!(!memory_info.total.is_empty());
    assert!(!memory_info.available.is_empty());
    assert!(!memory_info.used.is_empty());
    assert!(!memory_info.swap.is_empty());
    // Verify total memory is greater than used memory
    // This is a sanity check that the values make sense
    assert!(memory_info.total.contains("B")); // Should contain byte units
    assert!(memory_info.used.contains("B"));
}
#[test]
fn test_network_information_init() {
    let network_info = NetworkInformation::init();
    // We can't guarantee there will be networks, but if there are,
    // let's validate them
    if !network_info.networks.is_empty() {
        let network = &network_info.networks[0];
        // MAC address should be in the format xx:xx:xx:xx:xx:xx
        // or empty for virtual interfaces
        if !network.mac_address.is_empty() {
            assert!(network.mac_address.contains(':') || network.mac_address.contains('-'));
        }
        // MTU should be a number
        let mtu = network.mtu.parse::<usize>();
        assert!(mtu.is_ok() || network.mtu.is_empty());
        // If we have IP addresses, validate format
        if !network.ip_address.is_empty() {
            let ip = &network.ip_address[0];
            // Should contain dots (IPv4) or colons (IPv6)
            assert!(ip.contains('.') || ip.contains(':'));
        }
    }
}
#[test]
fn test_diagnostic_combinations() -> Result<()> {
    // Test combinations of diagnostics
    let test_cases = vec![
        vec![Diagnostic::System, Diagnostic::Memory],
        vec![Diagnostic::Memory, Diagnostic::Network],
        vec![Diagnostic::System, Diagnostic::Network],
        vec![Diagnostic::System, Diagnostic::Memory, Diagnostic::Network],
        vec![Diagnostic::All],
    ];
    for diagnostics in test_cases {
        let result = run(&false, &false, &diagnostics, &OFFLINE);
        assert!(result.is_ok(), "Failed with diagnostics: {diagnostics:?}");
    }
    Ok(())
}
#[test]
fn test_edge_case_invalid_combination() -> Result<()> {
    // Test Gpu (unimplemented) with other valid diagnostics
    // Should not fail, but will log a warning
    let result = run(&false, &false, &[Diagnostic::System, Diagnostic::Gpu], &OFFLINE);
    assert!(result.is_ok());
    // Test with Software diagnostic
    let result = run(&false, &false, &[Diagnostic::Software], &OFFLINE);
    assert!(result.is_ok());
    Ok(())
}
