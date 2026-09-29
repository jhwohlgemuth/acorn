use super::*;
#[test]
fn injected_snapshots_are_deterministic_and_secret_free() {
    let snapshot = HostSnapshot::init()
        .architecture("test-arch")
        .cpu_count(4)
        .memory_available(512)
        .memory_total(1024)
        .network_interfaces(1)
        .os_name("TestOS")
        .software(vec![SoftwareSnapshot {
            name: "Vale".to_string(),
            path: None,
            version: None,
        }])
        .build();
    let first = serde_json::to_string(&snapshot.diagnose()).expect("serialize first snapshot");
    let second = serde_json::to_string(&snapshot.diagnose()).expect("serialize second snapshot");
    assert_eq!(first, second);
    assert!(!first.contains("host_name"));
}
