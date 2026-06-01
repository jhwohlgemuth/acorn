#![no_main]
use acorn::{Location, Repository};
use libfuzzer_sys::fuzz_target;

fn exercise_location(location: &Location) {
    let _ = location.scheme();
    let maybe_uri = location.uri();
    if maybe_uri.is_some() {
        let _ = location.hash();
    }
}
fn exercise_repository(repository: &Repository) {
    let _ = repository.is_local();
    let _ = repository.id();
    let _ = repository.raw_url("README.md".to_string());
    exercise_location(&repository.clone().location());
}
fuzz_target!(|data: &[u8]| {
    if let Ok(location) = serde_json::from_slice::<Location>(data) {
        exercise_location(&location);
    }

    if let Ok(repository) = serde_json::from_slice::<Repository>(data) {
        exercise_repository(&repository);
    }

    if let Ok(text) = core::str::from_utf8(data) {
        if let Ok(location) = serde_yml::from_str::<Location>(text) {
            exercise_location(&location);
        }

        if let Ok(repository) = serde_yml::from_str::<Repository>(text) {
            exercise_repository(&repository);
        }
    }
});
