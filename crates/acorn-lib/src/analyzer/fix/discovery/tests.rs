use super::*;
use crate::analyzer::fix::{DiscoveryCategory, DiscoveryStatus, IntoFixes, Plan, Policy, Runner, Verifier};
use alloc::{collections::BTreeMap, sync::Arc};
use core::sync::atomic::{AtomicUsize, Ordering};
use std::path::Path;
struct RecordingRunner;
#[derive(Clone)]
struct RecordingVerifier(Arc<AtomicUsize>);
impl Runner for RecordingRunner {
    fn run(&self, _binary: &str, _arguments: &[&str]) -> crate::io::ApiResult<String> {
        Ok(String::new())
    }
}
impl Verifier for RecordingVerifier {
    fn verify(&self, references: &[DiscoveryReference]) -> bool {
        self.0.store(references.len(), Ordering::Release);
        true
    }
}
fn reference(subject: &str, id: &str) -> DiscoveryReference {
    DiscoveryReference {
        category: DiscoveryCategory::Software,
        details: BTreeMap::new(),
        expected: "installed".to_string(),
        fingerprint: format!("{id:a<64}"),
        id: id.to_string(),
        observed: "missing".to_string(),
        status: DiscoveryStatus::Attention,
        subject: subject.to_string(),
    }
}
#[test]
fn node_family_coalesces_and_retains_all_references() {
    if cfg!(target_os = "macos") || cfg!(windows) {
        let report = HostProvider.provide(&[reference("Node.js", "node"), reference("npm", "npm"), reference("npx", "npx")]);
        let fixes = report.fixes().into_iter().cloned().collect::<Vec<_>>();
        let plan = Plan::from_fixes(&fixes, Policy::confirmed(false)).expect("plan node remediation");
        assert_eq!(plan.preview().len(), 1);
        assert_eq!(plan.references().len(), 3);
        let observed = Arc::new(AtomicUsize::new(0));
        plan.execute(Path::new("."), &RecordingRunner, RecordingVerifier(Arc::clone(&observed)))
            .expect("execute recorded remediation");
        assert_eq!(observed.load(Ordering::Acquire), 3);
    }
}
