use acorn_macros::With;

// Named struct happy path with Into conversion and chaining
#[derive(Debug, Clone, PartialEq, With)]
pub struct Config {
    host: String,
    port: u16,
    ttl: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, With)]
pub struct Tuple(String, u32);

#[derive(Debug, Clone, PartialEq, With)]
pub struct Generic<T> {
    value: T,
}

#[derive(Debug, Clone, PartialEq, With)]
pub struct Bounded<T>
where
    T: Clone,
{
    value: T,
}

#[derive(Debug, Clone, PartialEq, With)]
#[with(prefix = "set_")]
pub struct Prefixed {
    host: String,
}

#[derive(Debug, Clone, PartialEq, With)]
pub struct WithSkip {
    keep: String,
    #[with(skip)]
    skipped: String,
}

#[derive(Debug, Clone, PartialEq, With)]
pub struct Raw {
    #[allow(dead_code)]
    r#type: String,
}

#[derive(Debug, Clone, PartialEq, With)]
pub struct Unit;

#[derive(Debug, Clone, PartialEq, With)]
pub struct TupleWithSkip(String, #[with(skip)] u32, String);

#[derive(Debug, Clone, PartialEq, With)]
pub struct WithLifetime<'a> {
    value: &'a str,
}

#[test]
fn named_with_generates_setters_and_chaining() {
    let c = Config {
        host: "a".into(),
        port: 80,
        ttl: None,
    };
    let c2 = c.clone().with_host("example.com").with_port(8080u16).with_ttl(Some(10u32));
    assert_eq!(c2.host, "example.com");
    assert_eq!(c2.port, 8080);
    assert_eq!(c2.ttl, Some(10));
    // Into: &str -> String
    let c3 = c.with_host("other");
    assert_eq!(c3.host, "other");
}

#[test]
fn tuple_with_uses_index_naming() {
    let t = Tuple("a".into(), 1).with_0("b").with_1(2u32);
    assert_eq!(t, Tuple("b".into(), 2));
    // Into
    let t2 = Tuple("a".into(), 1).with_0("c");
    assert_eq!(t2.0, "c");
}

#[test]
fn generic_with_preserves_type() {
    let g = Generic { value: "a".to_string() }.with_value("b");
    assert_eq!(g.value, "b");
    let g2 = Generic { value: 1u32 }.with_value(2u32);
    assert_eq!(g2.value, 2);
}

#[test]
fn generic_with_preserves_where_clause() {
    let value = Bounded { value: String::from("a") }.with_value("b");
    assert_eq!(value.value, "b");
}

#[test]
fn prefix_customizes_method_name() {
    let p = Prefixed { host: "a".into() }.set_host("b");
    assert_eq!(p.host, "b");
}

#[test]
fn skip_omits_setter() {
    let s = WithSkip {
        keep: "a".into(),
        skipped: "x".into(),
    }
    .with_keep("b");
    assert_eq!(s.keep, "b");
    assert_eq!(s.skipped, "x");
}

#[test]
fn raw_ident_sanitized() {
    let r = Raw { r#type: "a".into() }.with_type("b");
    assert_eq!(r.r#type, "b");
}

#[test]
fn unit_has_no_setters_but_compiles() {
    let _u = Unit;
}

#[test]
fn tuple_skip_omits_index() {
    let t = TupleWithSkip("a".into(), 1, "c".into()).with_0("b").with_2("d");
    assert_eq!(t, TupleWithSkip("b".into(), 1, "d".into()));
}

#[test]
fn lifetime_struct_with() {
    let v = WithLifetime { value: "a" }.with_value("b");
    assert_eq!(v.value, "b");
}
