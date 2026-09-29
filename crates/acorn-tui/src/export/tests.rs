use super::*;
use acorn::analyzer::host::discovery::Records;
#[test]
fn export_uses_the_shared_record_schema() {
    let records = Records::diagnose_host();
    let exported = export_doctor_report(&records, None).expect("export records");
    let expected = serde_json::to_string_pretty(&records).expect("serialize records");
    assert_eq!(exported, expected);
}
