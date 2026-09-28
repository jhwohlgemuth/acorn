use super::*;

#[test]
fn test_api_options_default_domain_from_extension() {
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    struct MyExt;
    impl ApiExtension for MyExt {
        fn default_domain() -> String {
            "example.com".into()
        }
        fn env_token_var() -> &'static str {
            "EXAMPLE_TOKEN"
        }
        fn env_domain_var() -> &'static str {
            "EXAMPLE_HOST"
        }
    }
    let opts = ApiOptions::<MyExt>::with_token("secret").build();
    assert_eq!(opts.domain, "example.com");
    assert!(!format!("{opts:?}").contains("secret"));
}
#[test]
fn test_common_runtime_defaults() {
    let rt = CommonRuntime::default();
    assert!(!rt.offline);
    assert!(!rt.quiet);
    assert_eq!(rt.threads, 10);
}
#[test]
fn test_common_runtime_deserialization_preserves_defaults() {
    let rt: CommonRuntime = serde_json::from_str(r#"{"quiet":true}"#).expect("deserialize common runtime");
    assert!(rt.quiet);
    assert_eq!(rt.threads, 10);
}
#[test]
fn test_runtime_options_flatten_extension() {
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    struct Ext {
        #[serde(default)]
        extra: String,
    }
    let opts = RuntimeOptions::<Ext>::init()
        .common(CommonRuntime {
            offline: true,
            ..Default::default()
        })
        .extension(Ext { extra: "hello".into() })
        .build();
    let json = serde_json::to_string(&opts).expect("serialize");
    assert!(json.contains("hello"));
    assert!(json.contains("offline"));
    let back: RuntimeOptions<Ext> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.extension.extra, "hello");
    assert!(back.common.offline);
}
#[test]
fn test_sync_config_merges_target_paths_and_inner_values() {
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    struct Inner {
        value: Option<String>,
    }
    impl SyncInner for Inner {
        fn merge(self, overrides: Self) -> Self {
            Self {
                value: overrides.value.or(self.value),
            }
        }
    }
    let base = SyncConfig::<Inner> {
        goose: Some(TargetConfig {
            path: Some("base.yaml".into()),
            inner: Inner { value: Some("base".into()) },
        }),
        ..Default::default()
    };
    let overrides = SyncConfig::<Inner> {
        goose: Some(TargetConfig {
            path: Some("override.yaml".into()),
            inner: Inner {
                value: Some("override".into()),
            },
        }),
        ..Default::default()
    };
    let merged = base.merge(overrides);
    let goose = merged.goose.expect("merged target");
    assert_eq!(goose.path.as_deref(), Some("override.yaml"));
    assert_eq!(goose.value.as_deref(), Some("override"));
}
#[test]
fn test_target_config_flatten_inner() {
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    struct Inner {
        #[serde(default)]
        host: String,
    }
    impl SyncInner for Inner {}
    let cfg = TargetConfig::<Inner> {
        path: Some("path.yaml".into()),
        inner: Inner { host: "localhost".into() },
    };
    let json = serde_json::to_string(&cfg).expect("serialize");
    assert!(json.contains("localhost"));
    // path is skip_serializing so not in json (expected)
    assert!(!json.contains("path.yaml"));
}
