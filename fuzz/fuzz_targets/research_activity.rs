#![no_main]
use acorn_schema::schema::research_activity::ResearchActivity;
use acorn_core::util::ToProse;
use acorn_schema::validation::Validate;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(rad) = serde_json::from_slice::<ResearchActivity>(data) {
        let _ = rad.validate();
        let _ = rad.format().to_prose();
    }
    if let Ok(text) = core::str::from_utf8(data) {
        // YAML parser internals can panic on crafted inputs in edge cases.
        // Suppress the libfuzzer_sys abort hook so catch_unwind can intercept.
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result = std::panic::catch_unwind(|| serde_norway::from_str::<ResearchActivity>(text));
        std::panic::set_hook(prev);
        if let Ok(Ok(rad)) = result {
            let _ = rad.validate();
            let _ = rad.format().to_prose();
        }
    }
});
