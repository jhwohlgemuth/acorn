use acorn_macros::With;

mod visibility {
    use acorn_macros::With;

    #[derive(With)]
    pub struct Explicit {
        #[with(private)]
        hidden: bool,
        #[with(public, some)]
        label: Option<String>,
    }

    impl Explicit {
        pub fn new() -> Self {
            Self { hidden: false, label: None }
        }
        pub fn reveal(self) -> Self {
            self.with_hidden(true)
        }
        pub fn values(&self) -> (bool, Option<&str>) {
            (self.hidden, self.label.as_deref())
        }
    }

    #[derive(With)]
    pub struct Inherited {
        value: String,
    }

    impl Inherited {
        pub fn new() -> Self {
            Self { value: String::new() }
        }
        pub fn value(&self) -> &str {
            &self.value
        }
    }
}

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
pub enum Enum {
    Named {
        value: String,
        #[with(skip)]
        fixed: u32,
    },
    Unit,
}

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

#[derive(Debug, Clone, PartialEq, With)]
pub struct WithBorrowedSome<'a> {
    #[with(some)]
    value: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, With)]
pub struct WithQualifiedSome {
    #[with(some)]
    value: core::option::Option<String>,
}

#[derive(Debug, Clone, PartialEq, With)]
pub struct WithSome<T> {
    #[with(some)]
    value: Option<T>,
}

#[derive(Debug, Clone, PartialEq, With)]
pub struct TupleWithSome(#[with(some)] Option<String>);

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
fn enum_with_updates_named_variants_and_preserves_other_variants() {
    let named = Enum::Named {
        value: "before".to_string(),
        fixed: 1,
    }
    .with_value("after");
    assert_eq!(
        named,
        Enum::Named {
            value: "after".to_string(),
            fixed: 1
        }
    );
    assert_eq!(Enum::Unit.with_value("unused"), Enum::Unit);
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

#[test]
fn borrowed_some_preserves_reference_coercion() {
    let source = String::from("configured");
    let value = WithBorrowedSome { value: None }.with_value(&source);
    assert_eq!(value.value, Some("configured"));
}

#[test]
fn explicit_visibility_overrides_struct_visibility() {
    let options = visibility::Explicit::new().with_label("visible").reveal();
    assert_eq!(options.values(), (true, Some("visible")));
}

#[test]
fn some_accepts_values_convertible_to_the_option_inner_type() {
    let value = WithSome::<String> { value: None }.with_value("configured");
    assert_eq!(value.value.as_deref(), Some("configured"));
}

#[test]
fn some_supports_qualified_option_paths() {
    let value = WithQualifiedSome { value: None }.with_value("configured");
    assert_eq!(value.value.as_deref(), Some("configured"));
}

#[test]
fn tuple_some_wraps_the_converted_value() {
    let value = TupleWithSome(None).with_0("configured");
    assert_eq!(value.0.as_deref(), Some("configured"));
}

#[test]
fn visibility_defaults_to_struct_visibility() {
    let value = visibility::Inherited::new().with_value("configured");
    assert_eq!(value.value(), "configured");
}
