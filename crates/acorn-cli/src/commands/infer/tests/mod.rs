use super::resolve_prompt;
use core::cell::Cell;

#[test]
fn test_resolve_prompt_prefers_the_argument() {
    let read = Cell::new(false);
    let prompt = resolve_prompt(Some(" argument "), || {
        read.set(true);
        None
    })
    .expect("argument prompt");
    assert_eq!(prompt, "argument");
    assert!(!read.get());
}
#[test]
fn test_resolve_prompt_reads_standard_input() {
    let prompt = resolve_prompt(None, || Some(" piped prompt\n".to_string())).expect("piped prompt");
    assert_eq!(prompt, "piped prompt");
}
#[test]
fn test_resolve_prompt_rejects_missing_input() {
    assert!(resolve_prompt(None, || None).is_err());
    assert!(resolve_prompt(None, || Some(" \n".to_string())).is_err());
}
