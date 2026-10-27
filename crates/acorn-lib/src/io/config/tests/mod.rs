#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::*;
use crate::io::config::application;
use crate::{
    io::{
        api::{Authenticator, Credential},
        config::{
            bucket::source::{
                BucketSourceAdapter, BucketSourceIdentity, BucketSourceItem, BucketSourceKind, DownloadOptions, RepositoryBucketSource,
            },
            transfer::{count_image_files, count_json_files, has_image_extension, operations_complete_message, transfer_bucket_files, TransferItem},
        },
        read_file, InputOutput, Parse,
    },
    util::constants::app::{
        DEFAULT_CONFIG_FILENAMES, DEFAULT_INTAKE_MAX_FILE, DEFAULT_INTAKE_MAX_RECORD, DEFAULT_INTAKE_PATH, DEFAULT_INTAKE_POLL_INTERVAL_SECS,
        GITLAB_CREDENTIAL_VARIABLE_PAIRS,
    },
};
use acorn_core::time::{Seconds, UnitOfTime};
use acorn_core::{options::CommonRuntime, Location, OneOrMany, Repository};
use acorn_host::fs::{FsError, SafePath};
use acorn_schema::hardware::memory::Memory;
use acorn_schema::validation::Validate;
use alloc::collections::BTreeSet;
use fancy_regex::Regex;
use secrecy::ExposeSecret;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::{
    fs::{create_dir_all, read, read_to_string, remove_dir_all, write},
    path::{Path, PathBuf},
    sync::Mutex,
};

const ENDPOINTS: &str = r#"{
    "endpoints": [
        {
            "name": "orcid",
            "domain": "pub.orcid.org",
            "root": "v3.0",
            "resources": [
                {
                    "name": "search",
                    "method": "get",
                    "template": "{{ base }}/expanded-search/{{ query }}"
                },
                {
                    "name": "status",
                    "method": "get",
                    "template": "{{ base }}/pubStatus"
                }
            ]
        },
        {
            "name": "ror",
            "domain": "api.ror.org",
            "resources": [
                {
                    "name": "status",
                    "method": "get",
                    "template": "{{ base }}/heartbeat"
                }
            ]
        },
        {
            "name": "github",
            "domain": "api.github.com",
            "resources": [
                {
                    "name": "tree",
                    "method": "get",
                    "template": "{{ base }}/repos/{{ path }}/git/trees/{{ branch }}{{ query }}"
                },
                {
                    "name": "languages",
                    "method": "get",
                    "template": "{{ base }}/repos/{{ path }}/languages"
                }
            ]
        },
        {
            "name": "gitlab",
            "domain": "code.ornl.gov",
            "root": "api/v4",
            "resources": [
                {
                    "name": "tree",
                    "method": "get",
                    "template": "{{ base }}/projects/{{ project_id }}//repository/tree/{{ query }}"
                }
            ]
        },
        {
            "name": "languages::gitlab",
            "domain": "gitlab.com",
            "resources": [
                {
                    "name": "languages",
                    "method": "get",
                    "template": "{{ base }}/gitlab-org/gitlab/-/raw/master/vendor/languages.yml"
                }
            ]
        },
        {
            "name": "languages::github",
            "domain": "raw.githubusercontent.com",
            "resources": [
                {
                    "name": "languages",
                    "method": "get",
                    "template": "{{ base }}/github-linguist/linguist/refs/heads/main/lib/linguist/languages.yml"
                }
            ]
        },
        {
            "name": "spdx",
            "domain": "spdx.org",
            "resources": [
                {
                    "name": "licenses",
                    "method": "get",
                    "template": "{{ base }}/licenses/licenses.json"
                }
            ]
        }
    ]
}"#;
#[derive(Clone)]
struct FakeBucketSource {
    items: Vec<BucketSourceItem>,
}
impl BucketSourceAdapter for FakeBucketSource {
    fn is_local(&self) -> bool {
        false
    }
    async fn items(&self) -> ApiResult<Vec<BucketSourceItem>> {
        Ok(self.items.clone())
    }
    async fn retrieve(&self, item: &BucketSourceItem, destination: &Path) -> ApiResult<()> {
        write(destination, item.key()).map(|_| ()).map_err(Into::into)
    }
    fn source_identity(&self) -> BucketSourceIdentity {
        BucketSourceIdentity::new(BucketSourceKind::Git, "https://user:secret@example.org/bucket?token=secret")
    }
}
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}
fn source_items(paths: &[&str]) -> Vec<BucketSourceItem> {
    paths
        .iter()
        .map(|path| BucketSourceItem::new((*path).to_string(), path, None, None).unwrap())
        .collect()
}
fn temp_resolve_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(core::time::Duration::from_nanos(0))
        .as_nanos();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("test_artifacts")
        .join(format!("{name}-{nanos}"))
}

#[test]
fn test_acre_folder_defaults_and_precedence() {
    let configuration = ApplicationConfiguration::parse(
        r#"{"folders":[{"id":"pilot","aliases":"acorn"},{"id":"standalone","path":"file:///srv/research/standalone/index.json"}],"options":{"folders":{"root":"file:///srv/research/"}}}"#,
    )
    .expect("parse folders");
    let registry = configuration.registry();
    assert_eq!(
        registry.resolve_folder("acorn").expect("alias routing").destination,
        Destination::Local(std::path::PathBuf::from("/srv/research/pilot/index.json"))
    );
    assert_eq!(
        registry.resolve_folder("standalone").expect("absolute override").destination,
        Destination::Local(std::path::PathBuf::from("/srv/research/standalone/index.json"))
    );
    let mut unsafe_path = configuration;
    unsafe_path.folders = Some(OneOrMany::One(Folder {
        id: "pilot".to_string(),
        path: Some("../escape/index.json".to_string()),
        ..Folder::default()
    }));
    assert!(unsafe_path.validate().is_err());
}
#[test]
fn test_acre_folder_fixtures_resolve_across_formats() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/config");
    let json = ApplicationConfiguration::read(root.join("with_folders.json")).expect("read folder JSON fixture");
    let yaml = ApplicationConfiguration::read(root.join("with_folders.yaml")).expect("read folder YAML fixture");
    let jsonc = ApplicationConfiguration::read(root.join("with_folders.jsonc")).expect("read folder JSONC fixture");
    assert!(json.validate().is_ok());
    assert!(yaml.validate().is_ok());
    assert!(jsonc.validate().is_ok());
    assert_eq!(json.folders().count(), 2);
    assert_eq!(yaml.folders().count(), json.folders().count());
    assert_eq!(yaml.folder("pilot").expect("yaml folder").id, "pilot");
    assert_eq!(yaml.folder("acorn").expect("yaml alias").id, json.folder("acorn").expect("json alias").id);
    assert_eq!(yaml.variable("power_automate.callback_url"), Some("POWER_AUTOMATE_CALLBACK_URL"));
    assert_eq!(
        yaml.folder("standalone").expect("scalar alias").aliases.as_ref().map(OneOrMany::len),
        Some(1)
    );
    assert_eq!(
        yaml.resolve_folder("standalone").expect("absolute override").destination,
        Destination::Local(PathBuf::from("/srv/research/standalone/index.json"))
    );
    assert_eq!(
        yaml.resolve_folder("pilot").expect("root-relative default").destination,
        Destination::Local(PathBuf::from("/srv/research/pilot/index.json"))
    );
    let variables = |name: &str| match name {
        | "ACORN_PILOT_MEMBERS" => Some(" Scientist@example.org, reviewer@example.org ".to_string()),
        | _ => None,
    };
    let registry = json.registry();
    assert_eq!(
        registry.effective_members(Some("pilot"), &variables),
        BTreeSet::from([
            "office@example.org".to_string(),
            "reviewer@example.org".to_string(),
            "scientist@example.org".to_string()
        ])
    );
    assert!(registry
        .authorizes("pilot", "REVIEWER@example.org", &variables)
        .expect("variable-sourced member"));
    assert!(registry
        .authorizes("standalone", "reviewer@example.org", &variables)
        .expect("document-root variables.members reaches every configured folder"));
    assert!(registry
        .authorizes("standalone", "operator@example.org", &variables)
        .expect("inline folder member"));
    assert!(!registry
        .authorizes("pilot", "operator@example.org", &variables)
        .expect("folder members stay folder-scoped"));
    assert!(!registry
        .authorizes("pilot", "stranger@example.org", &variables)
        .expect("unlisted submitter"));
    assert_eq!(jsonc.registry().effective_members(Some("pilot"), &variables).len(), 2);
}
#[cfg(feature = "analysis")]
#[test]
fn test_acre_implements_portable_decode_contract() {
    let decoded = <ApplicationConfiguration as acorn_schema::codec::DocumentCodec>::decode("{}", &acorn_core::util::MimeType::Json);
    assert!(decoded.is_ok());
    let unsupported = <ApplicationConfiguration as acorn_schema::codec::DocumentCodec>::decode("{}", &acorn_core::util::MimeType::Markdown);
    assert!(unsupported.is_err());
}
#[test]
fn test_acre_intake_fixture_parses_validates_and_resolves() {
    let configuration =
        ApplicationConfiguration::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/automation/powerautomate/intake.json"))
            .expect("read intake fixture");
    assert!(configuration.validate().is_ok());
    let intake = configuration
        .power_automate()
        .expect("PowerAutomate options")
        .intake
        .clone()
        .expect("intake source");
    assert_eq!(intake.branch, "acorn/powerautomate-pilot");
    assert_eq!(intake.path, ".acorn/intake/logbook.jsonl");
    assert_eq!(intake.poll_interval, Seconds::new(30));
    assert_eq!(intake.max_file, Memory::mb(10));
    assert_eq!(intake.max_record, Memory::kb(64));
    let registry = configuration.registry();
    assert_eq!(registry.folder("acorn-tooling").expect("folder alias").id, "acorn");
    assert_eq!(registry.folder("rad").expect("folder alias").id, "research-data");
    assert_eq!(
        registry.resolve_folder("research-data").expect("omitted path default").destination,
        Destination::Local(PathBuf::from("/srv/research/research-data/index.json"))
    );
}
#[test]
fn test_acre_options_scripts_owns_both_root_attributes() {
    let options = serde_json::from_value::<ApplicationOptions>(serde_json::json!({
        "crosswalk": { "localIdentifier": "imported-record" },
        "folders": { "root": "file:///srv/research/" },
        "powerAutomate": { "callback_destination": "pilot-flow", "gitlab_project": "42", "package": "automated-artifacts" },
        "scripts": { "enabled": false }
    }))
    .expect("parse options");
    assert!(!options.scripts.as_ref().expect("script options").enabled);
    assert!(options.folders.is_some());
    assert!(options.crosswalk.is_some());
    assert_eq!(UNGOVERNED_ROOT_ATTRIBUTES.len(), 3);
    let schema = serde_json::to_value(schemars::schema_for!(ApplicationOptions)).expect("options schema");
    let properties = schema["properties"].as_object().expect("options schema properties");
    ["crosswalk", "folders", "powerAutomate", "scripts"].into_iter().for_each(|name| {
        assert!(properties.contains_key(name), "missing options.{name}");
    });
    SCRIPT_OPTION_ROOTS.into_iter().for_each(|root| {
        assert!(SCRIPT_OPTION_ROOTS.contains(&root));
        assert!(!UNGOVERNED_ROOT_ATTRIBUTES.contains(&root), "{root} must be governed");
    });
    assert!(serde_json::from_value::<ApplicationOptions>(serde_json::json!({ "removed": true })).is_err());
    let defaults = ApplicationConfiguration::parse("{}").expect("parse default options");
    assert!(!defaults.has_hook(Hook::Check), "no root scripts are registered by default");
    let registered = ApplicationConfiguration::parse(r#"{"scripts":[{"path":"a.rhai","hooks":["check"]}]}"#).expect("parse scripts");
    assert!(registered.has_hook(Hook::Check));
    let disabled = ApplicationConfiguration::parse(r#"{"scripts":[{"path":"a.rhai","hooks":["check"]}],"options":{"scripts":{"enabled":false}}}"#)
        .expect("parse disabled scripts");
    assert!(!disabled.has_hook(Hook::Check), "options.scripts gates root scripts");
    let empty_object = ApplicationConfiguration::parse(r#"{"options":{"scripts":{}}}"#).expect("parse empty script options");
    assert_eq!(
        empty_object
            .options
            .as_ref()
            .and_then(|options| options.scripts.as_ref())
            .map(|scripts| scripts.enabled),
        Some(true),
        "an omitted enabled flag defaults to true"
    );
    assert!(serde_json::to_value(empty_object).expect("serialize options")["options"]["scripts"].is_object());
}
#[test]
fn test_acre_power_automate_intake_defaults_and_validation() {
    let minimal = ApplicationConfiguration::parse(
        r#"{"options":{"powerAutomate":{"callback_destination":"pilot-flow","gitlab_project":"42","package":"artifacts"}}}"#,
    )
    .expect("parse PowerAutomate options without intake");
    assert!(minimal.power_automate().expect("PowerAutomate options").intake.is_none());
    assert!(minimal.validate().is_ok());
    let partial = ApplicationConfiguration::parse(
        r#"{"options":{"powerAutomate":{"callback_destination":"pilot-flow","gitlab_project":"42","package":"artifacts","intake":{"branch":"pilot"}}}}"#,
    )
    .expect("parse partial intake");
    let intake = partial
        .power_automate()
        .expect("PowerAutomate options")
        .intake
        .clone()
        .expect("intake defaults");
    assert_eq!(intake.branch, "pilot");
    assert_eq!(intake.path, DEFAULT_INTAKE_PATH);
    assert_eq!(intake.poll_interval, Seconds::new(DEFAULT_INTAKE_POLL_INTERVAL_SECS));
    assert!(serde_json::from_str::<application::PowerAutomateIntake>(r#"{"branch":"main","poll_interval_secs":30}"#).is_err());
    assert!(serde_json::from_str::<application::PowerAutomateIntake>(r#"{"branch":"main","poll_interval":-1}"#).is_err());
    assert!(serde_json::from_str::<application::PowerAutomateIntake>(r#"{"branch":"main","poll_interval":0.5}"#).is_err());
    assert_eq!(intake.max_file, DEFAULT_INTAKE_MAX_FILE);
    assert_eq!(intake.max_record, DEFAULT_INTAKE_MAX_RECORD);
    assert!(partial.validate().is_ok());
    [
        r#"{"branch":"","path":".acorn/intake/logbook.jsonl","poll_interval":30,"max_file":"10MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":"","poll_interval":30,"max_file":"10MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":"file:///srv/research/logbook.jsonl","poll_interval":30,"max_file":"10MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":"/srv/research/logbook.jsonl","poll_interval":30,"max_file":"10MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":"C:/research/logbook.jsonl","poll_interval":30,"max_file":"10MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":"../escape.jsonl","poll_interval":30,"max_file":"10MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":"a/../../escape.jsonl","poll_interval":30,"max_file":"10MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":"a\\b.jsonl","poll_interval":30,"max_file":"10MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":".acorn/intake/logbook.jsonl","poll_interval":0,"max_file":"10MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":".acorn/intake/logbook.jsonl","poll_interval":86401,"max_file":"10MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":".acorn/intake/logbook.jsonl","poll_interval":30,"max_file":"0MB","max_record":"64KB"}"#,
        r#"{"branch":"main","path":".acorn/intake/logbook.jsonl","poll_interval":30,"max_file":"1KB","max_record":"0MB"}"#,
        r#"{"branch":"main","path":".acorn/intake/logbook.jsonl","poll_interval":30,"max_file":"1KB","max_record":"1MB"}"#,
    ]
    .into_iter()
    .for_each(|intake| {
        let content = format!(
            "{{\"options\":{{\"powerAutomate\":{{\"callback_destination\":\"pilot-flow\",\"gitlab_project\":\"42\",\"package\":\"artifacts\",\"intake\":{intake}}}}}}}"
        );
        let configuration = ApplicationConfiguration::parse(&content).expect("intake parses before validation");
        assert!(configuration.validate().is_err(), "accepted invalid intake {intake}");
    });
    let unknown = ApplicationConfiguration::parse(
        r#"{"options":{"powerAutomate":{"callback_destination":"pilot-flow","gitlab_project":"42","package":"artifacts","intake":{"branch":"main","unknown":true}}}}"#,
    );
    assert!(unknown.is_err(), "accepted unknown intake attribute");
}
#[test]
fn test_acre_rejects_unsupported_schema_versions_and_unknown_fields() {
    [
        r#"{"schemaVersion":"1.0.0"}"#,
        r#"{"schemaVersion":"v2.0.0"}"#,
        r#"{"schemaVersion":1}"#,
        r#"{"unknown":true}"#,
    ]
    .into_iter()
    .for_each(|content| assert!(ApplicationConfiguration::parse(content).is_err(), "accepted {content}"));
    let message = ApplicationConfiguration::parse(r#"{"schemaVersion":"v2.0.0"}"#)
        .expect_err("unknown ACRE version should fail")
        .to_string();
    assert!(message.contains("unknown variant `v2.0.0`"));
    assert!(!message.contains("missing field"));
}
#[test]
fn test_acre_resolves_bucket_references() {
    let configuration = ApplicationConfiguration::parse(
        r#"{"buckets":[{"name":"nssd","source":{"provider":"gitlab","uri":"https://code.ornl.gov/research-enablement/buckets/nssd"}}],"folders":[{"id":"pilot","bucket":0},{"id":"other","bucket":"nssd"}],"options":{"powerAutomate":{"callback_destination":"pilot-flow","gitlab_project":"42","package":"automated-artifacts"}}}"#,
    )
    .expect("parse bucket references");
    let folder = configuration.folder("pilot").expect("bucket index zero").clone();
    assert_eq!(folder.bucket, Some(BucketReference::Index(0)));
    assert_eq!(
        configuration.folder("other").expect("bucket identifier").bucket,
        Some(BucketReference::Identifier("nssd".to_string()))
    );
    assert!(configuration.validate().is_ok());
    let missing = ApplicationConfiguration::parse(r#"{"folders":[{"id":"pilot","bucket":3}]}"#).expect("parse out-of-range bucket");
    assert!(missing.validate().is_err());
}
#[test]
fn test_acre_schema_has_human_title_and_feature_independent_fields() {
    let schema = serde_json::to_value(schemars::schema_for!(ApplicationConfiguration)).unwrap();
    assert_eq!(schema["title"], "Application Configuration for Research Enablement");
    let properties = schema["properties"].as_object().expect("schema properties");
    ["folders", "mcp", "options", "schemaVersion", "scriptManifest", "variables", "workflow"]
        .into_iter()
        .for_each(|name| assert!(properties.contains_key(name), "missing {name}"));
    let bucket_properties = schema["$defs"]["Bucket"]["properties"].as_object().expect("bucket schema properties");
    assert!(bucket_properties.contains_key("source"));
    assert!(!bucket_properties.contains_key("codeRepository"));
    assert!(!bucket_properties.contains_key("repository"));
}
#[test]
fn test_acre_schema_version_defaults_and_serializes() {
    let configuration = ApplicationConfiguration::parse("{}").expect("parse minimal configuration");
    assert_eq!(configuration.schema_version, AcreSchemaVersion::V2_0_0);
    assert_eq!(serde_json::to_value(configuration).unwrap()["schemaVersion"], "2.0.0");
}
#[test]
fn test_acre_supports_crosswalk_import_defaults() {
    let configuration =
        ApplicationConfiguration::parse(r#"{"options":{"crosswalk":{"contact":"contacts/default.yaml","localIdentifier":"imported-record"}}}"#)
            .expect("parse crosswalk options");
    let options = configuration.options.and_then(|options| options.crosswalk).expect("crosswalk options");
    assert_eq!(options.contact.as_deref(), Some(Path::new("contacts/default.yaml")));
    assert_eq!(options.local_identifier.as_deref(), Some("imported-record"));
}
#[test]
fn test_acre_supports_power_automate_options_and_folders() {
    let configuration = ApplicationConfiguration::parse(
        r#"{"schemaVersion":"2.0.0","folders":{"id":"pilot","members":"scientist@example.org","path":"pilot/index.json"},"variables":{"custom":{"power_automate.inbound_secret":"POWER_AUTOMATE_INBOUND_SECRET"}},"options":{"powerAutomate":{"callback_destination":"pilot-flow","gitlab_project":"42","package":"automated-artifacts"}}}"#,
    )
    .expect("parse PowerAutomate options");
    let automation = configuration.power_automate().expect("PowerAutomate options");
    assert_eq!(automation.gitlab_project, "42");
    assert_eq!(
        configuration.variable("power_automate.inbound_secret"),
        Some("POWER_AUTOMATE_INBOUND_SECRET")
    );
    let registry = configuration.registry();
    assert_eq!(registry.effective_members(Some("pilot"), &|_| None).len(), 1);
    assert_eq!(
        registry.resolve_folder("pilot").expect("resolved folder").destination,
        Destination::GitLab {
            project: "42".to_string(),
            path: "pilot/index.json".to_string()
        }
    );
    assert!(configuration.validate().is_ok());
}
#[test]
fn test_auth_profiles_preserve_declaration_order_in_all_formats() {
    for (format, content) in [
        ("json", include_str!("../../../../../../tests/fixtures/oauth2/config/ordered.json")),
        ("jsonc", include_str!("../../../../../../tests/fixtures/oauth2/config/ordered.jsonc")),
        ("yaml", include_str!("../../../../../../tests/fixtures/oauth2/config/ordered.yaml")),
    ] {
        let configuration = ApplicationConfiguration::parse(content).expect("auth fixture parses");
        configuration.validate().expect("auth fixture validates");
        let auth = configuration.auth.as_ref().expect("auth attribute");
        assert_eq!(auth.profile(None).map(|(name, _)| name), Some("zeta"), "{format}");
        assert_eq!(auth.profile(Some("alpha")).map(|(name, _)| name), Some("alpha"));
        assert!(auth.profile(Some("missing")).is_none());
        let serialized = match format {
            | "yaml" => serde_norway::to_string(&configuration).expect("serialize YAML auth config"),
            | "jsonc" => configuration.cst.as_ref().expect("JSONC syntax tree").to_string(),
            | _ => serde_json::to_string(&configuration).expect("serialize JSON auth config"),
        };
        let reparsed = ApplicationConfiguration::parse(serialized).expect("parse serialized auth config");
        assert_eq!(
            reparsed.auth.as_ref().and_then(|auth| auth.profile(None)).map(|(name, _)| name),
            Some("zeta")
        );
    }
}
#[test]
fn test_auth_rejects_duplicate_profiles_and_scope_widening() {
    let duplicate = include_str!("../../../../../../tests/fixtures/oauth2/config/ordered.json").replace("\"alpha\":", "\"zeta\":");
    assert!(ApplicationConfiguration::parse(duplicate).is_err());
    let duplicate_resource = r#"{"schemaVersion":"2.0.0","auth":{"profiles":{"one":{"provider":"entra","tenantId":"00000000-0000-0000-0000-000000000001","clientId":"00000000-0000-0000-0000-000000000002","cloud":"public","resources":{"graph":{"audience":"https://graph.microsoft.com","scopes":["Files.Read"]},"graph":{"audience":"https://graph.microsoft.com","scopes":["Sites.Read.All"]}},"defaultResource":"graph"}},"sources":[]}}"#;
    assert!(ApplicationConfiguration::parse(duplicate_resource).is_err());
    let mut configuration =
        ApplicationConfiguration::parse(include_str!("../../../../../../tests/fixtures/oauth2/config/ordered.json")).expect("auth fixture parses");
    let auth = configuration.auth.as_mut().expect("auth attribute");
    auth.sources.push(application::auth::SharePointSource {
        drive_id: None,
        host: "example.sharepoint.com".to_string(),
        id: "combined".to_string(),
        item_id: None,
        item_path: "logbook.jsonl".to_string(),
        library_path: "/Library".to_string(),
        profile: "zeta".to_string(),
        provider: application::auth::SourceProvider::Sharepoint,
        resource: "graph".to_string(),
        scopes: Some(vec!["Sites.Read.All".to_string()]),
        site_id: None,
        site_path: "/sites/Example".to_string(),
    });
    assert!(configuration.validate().is_err());
}
#[test]
fn test_auth_rejects_embedded_secrets_invalid_ids_and_incomplete_pins() {
    let fixture = include_str!("../../../../../../tests/fixtures/oauth2/config/ordered.json");
    let embedded_secret = fixture.replace("\"provider\": \"entra\",", "\"provider\": \"entra\", \"clientSecret\": \"secret\",");
    assert!(ApplicationConfiguration::parse(embedded_secret).is_err());

    let invalid_id = fixture.replace("00000000-0000-0000-0000-000000000001", "not-a-tenant");
    let configuration = ApplicationConfiguration::parse(invalid_id).expect("invalid UUID parses for validation");
    assert!(configuration.validate().is_err());

    let mut configuration = ApplicationConfiguration::parse(fixture).expect("auth fixture parses");
    configuration
        .auth
        .as_mut()
        .expect("auth attribute")
        .sources
        .push(application::auth::SharePointSource {
            drive_id: None,
            host: "example.sharepoint.com".to_string(),
            id: "combined".to_string(),
            item_id: None,
            item_path: "logbook.jsonl".to_string(),
            library_path: "/Library".to_string(),
            profile: "zeta".to_string(),
            provider: application::auth::SourceProvider::Sharepoint,
            resource: "graph".to_string(),
            scopes: None,
            site_id: Some("site-id".to_string()),
            site_path: "/sites/Example".to_string(),
        });
    assert!(configuration.validate().is_err());
}
#[test]
fn test_auth_schema_exports_profile_and_source_fields() {
    let schema = schemars::schema_for!(ApplicationConfiguration);
    let json = serde_json::to_string(&schema).expect("ACRE schema");
    assert!(json.contains("\"auth\""));
    assert!(json.contains("\"profiles\""));
    assert!(json.contains("\"sources\""));
}
#[test]
fn test_auth_source_matches_one_exact_document_without_widening() {
    let mut configuration =
        ApplicationConfiguration::parse(include_str!("../../../../../../tests/fixtures/oauth2/config/ordered.json")).expect("auth fixture parses");
    let auth = configuration.auth.as_mut().expect("auth attribute");
    auth.sources.push(application::auth::SharePointSource {
        drive_id: None,
        host: "example.sharepoint.com".to_string(),
        id: "combined".to_string(),
        item_id: None,
        item_path: "logbook.jsonl".to_string(),
        library_path: "/Sponsor Highlights".to_string(),
        profile: "zeta".to_string(),
        provider: application::auth::SourceProvider::Sharepoint,
        resource: "graph".to_string(),
        scopes: Some(vec!["Files.Read".to_string()]),
        site_id: None,
        site_path: "/sites/Example".to_string(),
    });
    configuration.validate().expect("source policy");
    let auth = configuration.auth.as_ref().expect("auth attribute");
    assert_eq!(
        auth.source("https://example.sharepoint.com/sites/Example/Sponsor%20Highlights/logbook.jsonl")
            .expect("configured document")
            .id,
        "combined"
    );
    for url in [
        "https://example.sharepoint.com/sites/Example/Sponsor%20Highlights/other.jsonl",
        "https://example.sharepoint.com/sites/Example/Sponsor%20Highlights/logbook.jsonl?download=1",
        "https://example.sharepoint.com/sites/Example/Sponsor%2FHighlights/logbook.jsonl",
        "https://example.sharepoint.com/sites/Example/../Example/Sponsor%20Highlights/logbook.jsonl",
    ] {
        assert!(auth.source(url).is_err(), "{url}");
    }
    let mut duplicate = auth.sources.first().expect("configured source").clone();
    duplicate.id = "second".to_string();
    configuration.auth.as_mut().expect("auth attribute").sources.push(duplicate);
    assert!(configuration.validate().is_err());
    assert!(configuration
        .auth
        .as_ref()
        .expect("auth attribute")
        .source("https://example.sharepoint.com/sites/Example/Sponsor%20Highlights/logbook.jsonl")
        .is_err());
}
#[test]
fn test_bucket() {
    let bucket = Bucket {
        name: Some("nssd".to_string()),
        description: Some("Bucket for NSSD".to_string()),
        source: Repository::GitLab {
            id: Some(1234_u64),
            location: Box::new(Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string())),
        }
        .into(),
    };
    assert_eq!(bucket.domain().unwrap(), "code.ornl.gov".to_string());
    let bucket = Bucket {
        name: Some("nssd".to_string()),
        description: Some("Bucket for NSSD".to_string()),
        source: Repository::GitHub {
            location: Box::new(Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string())),
        }
        .into(),
    };
    assert_eq!(bucket.domain().unwrap(), "code.ornl.gov".to_string());
}
#[test]
fn test_bucket_config_json() {
    let path = fixtures_dir().join("config").join("with_github_buckets.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    assert_eq!(config.buckets.as_ref().map_or(0, |b| b.len()), 2);
    let path = fixtures_dir().join("config").join("with_remote_buckets.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    assert_eq!(config.buckets.as_ref().map_or(0, |b| b.len()), 3);
}
#[test]
fn test_bucket_config_jsonc() {
    let path = fixtures_dir().join("config").join("with_buckets.jsonc");
    let buckets = ApplicationConfiguration::read_jsonc(path).unwrap().buckets.unwrap();
    assert_eq!(buckets.len(), 1);
    assert_eq!(buckets[0].name.as_deref(), Some("test-bucket"));
}
#[test]
fn test_bucket_config_yaml() {
    let path = fixtures_dir().join("config").join("with_remote_buckets.yaml");
    let config = ApplicationConfiguration::read_yaml(path).unwrap();
    assert_eq!(config.buckets.as_ref().map_or(0, |b| b.len()), 3);
}
#[test]
fn test_bucket_legacy_aliases_normalize_and_serialize_to_source() {
    let repository = r#"{"provider":"git","location":"file:./science"}"#;
    let legacy = ApplicationConfiguration::parse(format!(r#"{{"buckets":[{{"repository":{repository}}}]}}"#).as_str()).unwrap();
    let canonical = ApplicationConfiguration::parse(format!(r#"{{"buckets":[{{"source":{repository}}}]}}"#).as_str()).unwrap();
    let legacy_bucket = &legacy.buckets.unwrap()[0];
    let canonical_bucket = &canonical.buckets.unwrap()[0];
    assert_eq!(legacy_bucket.source, canonical_bucket.source);
    assert_eq!(
        serde_json::to_value(legacy_bucket).unwrap(),
        serde_json::json!({
            "source": {
                "provider": "git",
                "location": "file:./science"
            }
        })
    );
}
#[test]
fn test_bucket_shorthand_classification_is_exact_and_fallible() {
    let local = Bucket::try_from("../science").unwrap();
    let github = Bucket::try_from("https://www.github.com/acme/science").unwrap();
    let lookalike = Bucket::try_from("https://github.com.example/acme/science").unwrap();
    assert!(Repository::parse("https://github.com/acme/science").unwrap().is_git_hub());
    assert!(matches!(local.source, BucketSource::Repository(Repository::Git { .. })));
    assert!(matches!(github.source, BucketSource::Repository(Repository::GitHub { .. })));
    assert!(matches!(lookalike.source, BucketSource::Repository(Repository::GitLab { id: None, .. })));
    [
        "",
        " https://github.com/acme/science",
        "http://github.com/acme/science",
        "ssh://git@github.com/acme/science",
        "https://user:secret@github.com/acme/science",
        "https://",
        "oci://registry.example/acme/science:latest",
    ]
    .into_iter()
    .for_each(|value| assert!(Bucket::try_from(value).is_err(), "{value} should be rejected"));
    let message = Bucket::try_from("https://user:secret@github.com/acme/science").unwrap_err().to_string();
    assert!(!message.contains("secret"));
}
#[test]
fn test_bucket_shorthand_normalizes_across_json_jsonc_and_yaml() {
    let json = ApplicationConfiguration::parse(r#"{"buckets":["./science",{"name":"remote","source":"https://github.com/acme/science"}]}"#).unwrap();
    let jsonc = ApplicationConfiguration::parse(
        r#"{
            // Concise bucket definitions use the same normalization path.
            "buckets": ["./science", {"name": "remote", "source": "https://github.com/acme/science"}],
        }"#,
    )
    .unwrap();
    let yaml = serde_norway::from_str::<ApplicationConfiguration>(
        "buckets:\n  - ./science\n  - name: remote\n    source: https://github.com/acme/science\n",
    )
    .unwrap();
    let expected = json.buckets.unwrap();
    [jsonc, yaml].into_iter().for_each(|configuration| {
        let buckets = configuration.buckets.unwrap();
        assert_eq!(buckets.len(), expected.len());
        assert_eq!(buckets[0].source, expected[0].source);
        assert_eq!(buckets[1].source, expected[1].source);
        assert_eq!(buckets[1].name, expected[1].name);
    });
    assert_eq!(
        serde_json::to_value(&expected[1]).unwrap(),
        serde_json::json!({
            "name": "remote",
            "source": {
                "provider": "github",
                "location": "https://github.com/acme/science"
            }
        })
    );
}
#[tokio::test]
async fn test_bucket_source_adapter_preserves_item_metadata_through_flattening() {
    let output = temp_resolve_dir("provider-neutral-manifest");
    create_dir_all(&output).unwrap();
    let adapter = FakeBucketSource {
        items: vec![
            BucketSourceItem::new(
                "opaque:item:42".to_string(),
                "nested/keep.json",
                Some("exact-version".to_string()),
                Some(17),
            )
            .unwrap(),
            BucketSourceItem::new("opaque:item:discard".to_string(), "nested/discard.md", None, None).unwrap(),
        ],
    };
    let options = BucketOptions::init()
        .common(CommonRuntime {
            filter: vec!["keep\\.json$".to_string()],
            quiet: true,
            ..CommonRuntime::default()
        })
        .extension(BucketExtension {
            flatten: true,
            ..BucketExtension::default()
        })
        .output(output.clone())
        .build();
    let manifest = transfer_bucket_files(Some("science".to_string()), &adapter, &options).await.unwrap();
    assert_eq!(manifest.bucket(), Some("science"));
    assert_eq!(manifest.source().kind(), BucketSourceKind::Git);
    assert_eq!(manifest.source().locator(), "https://example.org/bucket");
    assert_eq!(manifest.items().len(), 1);
    let item = &manifest.items()[0];
    assert_eq!(item.key(), "opaque:item:42");
    assert_eq!(item.output_path(), Path::new("keep.json"));
    assert_eq!(item.size(), Some(17));
    assert_eq!(item.source_path(), Path::new("nested/keep.json"));
    assert_eq!(item.version(), Some("exact-version"));
    assert_eq!(read_to_string(output.join("keep.json")).unwrap(), "opaque:item:42");
    let _ = remove_dir_all(output);
}
#[test]
fn test_bucket_source_aliases_conflict_instead_of_using_precedence() {
    let content = r#"{
        "buckets": [{
            "source": "./science",
            "repository": {"provider": "git", "location": "file:./other"}
        }]
    }"#;
    assert!(ApplicationConfiguration::parse(content).is_err());
}
#[test]
fn test_bucket_source_identity_removes_url_secrets() {
    let identity = BucketSourceIdentity::new(
        BucketSourceKind::GitLab,
        "https://user:password@example.org:8443/acme/project?token=secret#fragment",
    );
    assert_eq!(identity.locator(), "https://example.org:8443/acme/project");
}
#[test]
fn test_bucket_source_items_validate_paths_duplicates_and_order() {
    ["", "/absolute.json", "../outside.json", "C:\\outside.json"]
        .into_iter()
        .for_each(|path| assert!(BucketSourceItem::new("key".to_string(), path, None, None).is_err()));
    let ordered = source_items(&["second.json", "first.json"]);
    assert_eq!(
        BucketSourceItem::validate(ordered)
            .unwrap()
            .into_iter()
            .map(|item| item.path().clone().into_path_buf())
            .collect::<Vec<_>>(),
        vec![PathBuf::from("second.json"), PathBuf::from("first.json")]
    );
    let duplicates = vec![
        BucketSourceItem::new("first".to_string(), "docs/./index.json", None, None).unwrap(),
        BucketSourceItem::new("second".to_string(), "docs/index.json", None, None).unwrap(),
    ];
    assert!(BucketSourceItem::validate(duplicates).unwrap_err().to_string().contains("duplicate path"));
}
#[test]
fn test_bucket_source_kind_from_repository_variants() {
    let location = || Box::new(Location::from("https://example.org/acme/project"));
    [
        (Repository::Git { location: location() }, BucketSourceKind::Git),
        (Repository::GitHub { location: location() }, BucketSourceKind::GitHub),
        (
            Repository::GitLab {
                id: None,
                location: location(),
            },
            BucketSourceKind::GitLab,
        ),
        (Repository::HuggingFace { location: location() }, BucketSourceKind::HuggingFace),
        (
            Repository::OCI {
                location: location(),
                registry: None,
            },
            BucketSourceKind::OCI,
        ),
    ]
    .into_iter()
    .for_each(|(repository, expected)| assert_eq!(BucketSourceKind::from(repository), expected));
}
#[test]
fn test_builtin_endpoints() {
    let config = ApplicationConfiguration::parse(ENDPOINTS).unwrap();
    insta::assert_snapshot!("builtin_endpoints", format!("{:#?}", config.endpoints));
}
#[tokio::test]
async fn test_clobber_preserves_destination_when_source_read_fails() {
    let output = temp_resolve_dir("clobber-source-failure-output");
    create_dir_all(&output).unwrap();
    write(output.join("index.json"), "old").unwrap();
    let write_lock = Mutex::new(());
    let destination = SafePath::new("index.json").unwrap();
    let source = output.join("missing.json");
    let result = destination
        .write_file(&output, true, &write_lock, || async move {
            read(&source).map_err(|error| FsError::Io { path: source, source: error })
        })
        .await;
    assert!(result.is_err());
    assert_eq!(read_to_string(output.join("index.json")).unwrap(), "old");
    let _ = remove_dir_all(output);
}
#[test]
fn test_config_with_endpoints() {
    let path = fixtures_dir().join("config").join("with_endpoints.json");
    let endpoints = ApplicationConfiguration::read_json(path).unwrap().endpoints.unwrap();
    assert_eq!(endpoints[0].resources.len(), 2);
}
#[tokio::test]
async fn test_copy_files_clobbers_selected_path_conflicts_only() {
    let source = temp_resolve_dir("clobber-enabled-source");
    let output = temp_resolve_dir("clobber-enabled-output");
    create_dir_all(source.join("nested")).unwrap();
    create_dir_all(output.join("directory.json")).unwrap();
    write(source.join("existing.json"), "new file").unwrap();
    write(source.join("directory.json"), "new directory replacement").unwrap();
    write(source.join("nested/index.json"), "new nested file").unwrap();
    write(source.join("nested/other.json"), "new sibling file").unwrap();
    write(output.join("existing.json"), "old file").unwrap();
    write(output.join("directory.json/old.json"), "old directory content").unwrap();
    write(output.join("nested"), "old parent file").unwrap();
    write(output.join("unrelated.json"), "keep").unwrap();
    let bucket = Bucket::init()
        .source(
            Repository::Git {
                location: Box::new(Location::Simple(format!("file:{}", source.display()))),
            }
            .into(),
        )
        .build();
    let options = BucketOptions::init()
        .common(CommonRuntime {
            quiet: true,
            ..CommonRuntime::default()
        })
        .extension(BucketExtension {
            clobber: true,
            ..BucketExtension::default()
        })
        .output(output.clone())
        .build();
    assert!(bucket.copy_files(&options).await.is_ok());
    assert_eq!(read_to_string(output.join("existing.json")).unwrap(), "new file");
    assert_eq!(read_to_string(output.join("directory.json")).unwrap(), "new directory replacement");
    assert_eq!(read_to_string(output.join("nested/index.json")).unwrap(), "new nested file");
    assert_eq!(read_to_string(output.join("nested/other.json")).unwrap(), "new sibling file");
    assert_eq!(read_to_string(output.join("unrelated.json")).unwrap(), "keep");
    let _ = remove_dir_all(source);
    let _ = remove_dir_all(output);
}
#[tokio::test]
async fn test_copy_files_flattens_destinations_and_manifest() {
    let source = temp_resolve_dir("flatten-source");
    let output = temp_resolve_dir("flatten-output");
    create_dir_all(source.join("docs/quest")).unwrap();
    write(source.join("docs/quest/index.json"), "{}").unwrap();
    write(source.join("docs/quest/image.png"), "image").unwrap();
    let bucket = Bucket::init()
        .source(
            Repository::Git {
                location: Box::new(Location::Simple(format!("file:{}", source.display()))),
            }
            .into(),
        )
        .build();
    let options = BucketOptions::init()
        .common(CommonRuntime {
            quiet: true,
            ..CommonRuntime::default()
        })
        .extension(BucketExtension {
            flatten: true,
            ..BucketExtension::default()
        })
        .output(output.clone())
        .build();
    let manifest = bucket.copy_files(&options).await.unwrap();
    let files = manifest.files().map(Path::to_path_buf).collect::<Vec<_>>();
    assert_eq!(files.len(), 2);
    assert!(files.contains(&PathBuf::from("index.json")));
    assert!(files.contains(&PathBuf::from("image.png")));
    assert!(manifest.items().iter().all(|item| item.source_path().starts_with("docs/quest")));
    assert!(output.join("index.json").is_file());
    assert!(output.join("image.png").is_file());
    assert!(!output.join("docs").exists());
    let _ = remove_dir_all(source);
    let _ = remove_dir_all(output);
}
#[tokio::test]
async fn test_copy_files_preserves_existing_destination_without_clobber() {
    let source = temp_resolve_dir("clobber-disabled-source");
    let output = temp_resolve_dir("clobber-disabled-output");
    create_dir_all(&source).unwrap();
    create_dir_all(&output).unwrap();
    write(source.join("index.json"), "new").unwrap();
    write(output.join("index.json"), "old").unwrap();
    let bucket = Bucket::init()
        .source(
            Repository::Git {
                location: Box::new(Location::Simple(format!("file:{}", source.display()))),
            }
            .into(),
        )
        .build();
    let options = BucketOptions::init()
        .common(CommonRuntime {
            quiet: true,
            ..CommonRuntime::default()
        })
        .output(output.clone())
        .build();
    assert!(bucket.copy_files(&options).await.is_err());
    assert_eq!(read_to_string(output.join("index.json")).unwrap(), "old");
    let _ = remove_dir_all(source);
    let _ = remove_dir_all(output);
}
#[test]
fn test_count_image_files_counts_supported_extensions() {
    let paths = vec![
        "content/plot.png".to_string(),
        "content/photo.jpg".to_string(),
        "content/photo.jpeg".to_string(),
        "content/index.json".to_string(),
    ];
    assert_eq!(count_image_files(&paths), 2);
}
#[test]
fn test_count_json_files_counts_case_insensitive_json_paths() {
    let paths = vec![
        "content/index.json".to_string(),
        "content/README.md".to_string(),
        "content/data.JSON".to_string(),
    ];
    assert_eq!(count_json_files(&paths), 2);
}
#[test]
fn test_default_config_filename_precedence_is_preserved() {
    let directory = temp_resolve_dir("default-config-precedence");
    create_dir_all(&directory).expect("create configuration directory");
    DEFAULT_CONFIG_FILENAMES.iter().for_each(|name| {
        write(directory.join(name), "{}\n").expect("write configuration candidate");
    });
    assert_eq!(
        ApplicationConfiguration::resolve_in(&None, &directory),
        Some(directory.join(DEFAULT_CONFIG_FILENAMES[0]))
    );
    remove_dir_all(directory).expect("remove configuration directory");
}
#[test]
fn test_default_config_filenames_require_supported_extensions() {
    assert_eq!(DEFAULT_CONFIG_FILENAMES, [".acorn.json", ".acorn.jsonc", ".acorn.yaml", ".acorn.yml"]);
}
#[tokio::test]
async fn test_download_file_skips_existing_file_without_clobber() {
    let output = temp_resolve_dir("download-file-transfer");
    create_dir_all(&output).expect("temporary output should exist");
    write(output.join("data.txt"), "existing").expect("existing file should be writable");
    let repository = Repository::GitHub {
        location: Box::new(Location::Simple("https://github.com/acme/quest".to_string())),
    };
    let options = DownloadOptions {
        repository: &repository,
        output: &output,
        source: Path::new("data.txt"),
        destination: Path::new("data.txt"),
        version: None,
        size: None,
    };
    RepositoryBucketSource::download_file(&options)
        .await
        .expect("existing file should be skipped without network transfer");
    assert_eq!(
        read_to_string(output.join("data.txt")).expect("skipped destination should be readable"),
        "existing"
    );
    let _ = remove_dir_all(output);
}
#[test]
fn test_download_item_github_uses_exact_version_in_raw_url() {
    let repository = Repository::GitHub {
        location: Box::new(Location::Simple("https://github.com/acme/quest".to_string())),
    };
    let options = DownloadOptions {
        repository: &repository,
        output: Path::new(""),
        source: Path::new("index.json"),
        destination: Path::new("index.json"),
        version: Some("abc123"),
        size: None,
    };
    let item = RepositoryBucketSource::download_item(&options).expect("GitHub raw URL should build");
    assert_eq!(item.url, "https://raw.githubusercontent.com/acme/quest/abc123/index.json");
    assert!(item.headers.is_none());
}
#[test]
fn test_download_item_gitlab_defaults_to_head_and_percent_encodes_path() {
    let repository = Repository::GitLab {
        id: None,
        location: Box::new(Location::Simple("https://code.ornl.gov/group/project".to_string())),
    };
    let options = DownloadOptions {
        repository: &repository,
        output: Path::new(""),
        source: Path::new("content/data set/index.json"),
        destination: Path::new("index.json"),
        version: None,
        size: None,
    };
    let item = RepositoryBucketSource::download_item(&options).expect("GitLab raw URL should build");
    assert_eq!(
        item.url,
        "https://code.ornl.gov/api/v4/projects/group%2Fproject/repository/files/content%2Fdata%20set%2Findex.json/raw?ref=HEAD"
    );
}
#[tokio::test]
async fn test_download_item_gitlab_raw_url_uses_resolved_branch_and_encodes_path() {
    let repository = Repository::GitLab {
        id: None,
        location: Box::new(Location::Simple("https://code.ornl.gov/group/project".to_string())),
    };
    let options = DownloadOptions {
        repository: &repository,
        output: Path::new(""),
        source: Path::new("content/quest/index.json"),
        destination: Path::new("index.json"),
        version: Some("develop"),
        size: None,
    };
    let item = RepositoryBucketSource::download_item(&options).expect("GitLab raw URL should build");
    assert_eq!(
        item.url,
        "https://code.ornl.gov/api/v4/projects/group%2Fproject/repository/files/content%2Fquest%2Findex.json/raw?ref=develop"
    );
    assert_eq!(item.path, "index.json");
    assert!(item.size.is_none());
    match Credential::from_env(&GITLAB_CREDENTIAL_VARIABLE_PAIRS) {
        | Some(credential) => assert_eq!(
            item.headers
                .as_ref()
                .and_then(|headers| headers.get(credential.header))
                .and_then(|value| value.to_str().ok()),
            Some(ExposeSecret::expose_secret(&credential.token))
        ),
        | None => assert!(item.headers.is_none()),
    }
}
#[test]
fn test_filter_set_filter() {
    let selectors = vec![crate::io::Source::from("alpha.gguf"), crate::io::Source::from("beta.safetensors")];
    let filtered = FilterSet::filter(
        selectors,
        &["gguf$".to_string()],
        &["alpha".to_string()],
        crate::io::Source::identifier,
        |_| true,
    )
    .unwrap();
    assert!(filtered.is_empty());
}
#[test]
fn test_has_image_extension_matches_png_and_jpg() {
    assert!(has_image_extension(&"image.png".to_string()));
    assert!(has_image_extension(&"photo.JPG".to_string()));
    assert!(!has_image_extension(&"graphic.jpeg".to_string()));
}
#[test]
fn test_is_filtered_path() {
    let patterns = [r"\.json$".to_string(), r"img/".to_string()];
    let filter = FilterSet::try_from((&[][..], patterns.as_slice())).unwrap().filter;
    assert!(is_filtered_path("/tmp/data.json", &filter));
    assert!(is_filtered_path("/tmp/img/photo.jpg", &filter));
    assert!(!is_filtered_path("/tmp/README.md", &filter));
    let invalid = ["[".to_string()];
    assert!(FilterSet::try_from((&[][..], invalid.as_slice())).is_err());
    assert!(is_filtered_path("/tmp/README.md", &Vec::<Regex>::new()));
}
#[test]
fn test_is_ignored_path() {
    let patterns = [r"\.jpeg$".to_string(), r"notes\.txt$".to_string()];
    let ignore = FilterSet::try_from((patterns.as_slice(), &[][..])).unwrap().ignore;
    assert!(is_ignored_path("/tmp/photo.jpeg", &ignore));
    assert!(is_ignored_path("/tmp/notes.txt", &ignore));
    assert!(!is_ignored_path("/tmp/index.json", &ignore));
    let invalid = ["[".to_string()];
    assert!(FilterSet::try_from((invalid.as_slice(), &[][..])).is_err());
    assert!(is_ignored_path("/tmp/README.md", &Vec::<Regex>::new()));
}
#[test]
fn test_json_accepts_comments_via_jsonc_fallback() {
    let content = r#"{
        // This comment is valid via JSONC fallback
        "buckets": []
    }"#;
    assert!(ApplicationConfiguration::parse(content).is_ok());
}
#[test]
fn test_json_equivalent_to_jsonc() {
    let json = r#"{"buckets":[{"name":"t","repository":{"provider":"git","location":"file:./"}}]}"#;
    let jsonc = r#"{
        // comment
        "buckets": [{"name":"t","repository":{"provider":"git","location":"file:./"}},]
    }"#;
    let a = ApplicationConfiguration::parse(json).unwrap();
    let b = ApplicationConfiguration::parse(jsonc).unwrap();
    assert_eq!(a.buckets.as_ref().unwrap().len(), b.buckets.as_ref().unwrap().len());
}
#[test]
fn test_jsonc_parse_with_block_comments() {
    let content = r#"{
        /* Block comment */
        "buckets": []
    }"#;
    assert!(ApplicationConfiguration::parse(content).unwrap().buckets.unwrap().is_empty());
}
#[test]
fn test_jsonc_parse_with_line_comments() {
    let content = r#"{
        // This is a comment
        "buckets": [{"name": "test", "repository": {"provider": "git", "location": "file:./"}}]
    }"#;
    let buckets = ApplicationConfiguration::parse(content).unwrap().buckets.unwrap();
    assert_eq!(buckets.len(), 1);
}
#[test]
fn test_jsonc_parse_with_trailing_commas() {
    let content = r#"{
        "buckets": [],
    }"#;
    assert!(ApplicationConfiguration::parse(content).unwrap().buckets.unwrap().is_empty());
}
#[test]
fn test_load_dotenv_never_overrides_process_variables() {
    let directory = temp_resolve_dir("intake-dotenv-precedence");
    create_dir_all(&directory).unwrap();
    write(
        directory.join(".acorn.json"),
        r#"{"variables":{"members":"ACORN_PA20_INTAKE_TEST_MEMBERS"},"folders":[{"id":"pilot","path":"file:///srv/research/pilot/index.json"}]}"#,
    )
    .unwrap();
    write(directory.join(".env"), "ACORN_PA20_INTAKE_TEST_MEMBERS=file@example.org\n").unwrap();
    temp_env::with_var("ACORN_PA20_INTAKE_TEST_MEMBERS", Some("process@example.org"), || {
        let configuration = ApplicationConfiguration::load(&Some(directory.join(".acorn.json"))).expect("load with dotenv");
        assert!(configuration
            .registry()
            .authorizes("pilot", "process@example.org", &|name| std::env::var(name).ok())
            .expect("process-sourced member"));
        assert!(!configuration
            .registry()
            .authorizes("pilot", "file@example.org", &|name| std::env::var(name).ok())
            .expect("file value must not override process"));
    });
    let _ = remove_dir_all(directory);
}
#[test]
fn test_load_dotenv_next_to_config_supplies_member_lookup() {
    let directory = temp_resolve_dir("intake-dotenv");
    create_dir_all(&directory).unwrap();
    write(
        directory.join(".acorn.json"),
        r#"{"members":["office@example.org"],"variables":{"members":"ACORN_PA20_INTAKE_TEST_MEMBERS"},"folders":[{"id":"pilot","path":"file:///srv/research/pilot/index.json"}]}"#,
    )
    .unwrap();
    write(directory.join(".env"), "ACORN_PA20_INTAKE_TEST_MEMBERS=scientist@example.org\n").unwrap();
    temp_env::with_var("ACORN_PA20_INTAKE_TEST_MEMBERS", None::<&str>, || {
        let configuration = ApplicationConfiguration::load(&Some(directory.join(".acorn.json"))).expect("load with dotenv");
        assert!(configuration
            .registry()
            .authorizes("pilot", "scientist@example.org", &|name| std::env::var(name).ok())
            .expect("dotenv-sourced member"));
        assert!(!configuration
            .registry()
            .authorizes("pilot", "stranger@example.org", &|name| std::env::var(name).ok())
            .expect("unlisted submitter"));
    });
    let _ = remove_dir_all(directory);
}
#[test]
fn test_load_rejects_missing_explicit_path() {
    let missing = temp_resolve_dir("load-missing").join("missing.json");
    let result = ApplicationConfiguration::load(&Some(missing.clone()));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        format!("Configuration file does not exist — {}", missing.display())
    );
}
#[test]
fn test_load_without_dotenv_succeeds_and_rejects_malformed_dotenv() {
    let directory = temp_resolve_dir("intake-dotenv-missing");
    create_dir_all(&directory).unwrap();
    write(
        directory.join(".acorn.json"),
        r#"{"folders":[{"id":"pilot","path":"file:///srv/research/pilot/index.json"}]}"#,
    )
    .unwrap();
    temp_env::with_var("ACORN_PA20_INTAKE_TEST_MEMBERS", None::<&str>, || {
        assert!(ApplicationConfiguration::load(&Some(directory.join(".acorn.json"))).is_ok());
    });
    write(directory.join(".env"), "=missing-key\n").unwrap();
    temp_env::with_var("ACORN_PA20_INTAKE_TEST_MEMBERS", None::<&str>, || {
        let result = ApplicationConfiguration::load(&Some(directory.join(".acorn.json")));
        assert!(result.is_err(), "accepted malformed dotenv");
        assert!(result.expect_err("malformed dotenv").to_string().contains("deployment environment"));
    });
    let _ = remove_dir_all(directory);
}
#[test]
fn test_mcp_client_configuration_parses_and_validates_supported_formats() {
    let json = r#"{
        "mcp": {
            "research": {
                "transport": {"type": "child-process", "command": "research-mcp", "args": ["serve"]},
                "allowed": {"resolve": {"effects": ["network-read"], "timeout": 20, "max_output": "256KB"}}
            }
        }
    }"#;
    let jsonc = r#"{
        // Local development server
        "mcp": {
            "research": {
                "transport": {"type": "streamable-http", "url": "http://localhost:3000/mcp"},
                "allowed": {"resolve": {}}
            }
        }
    }"#;
    let yaml = r#"
mcp:
  research:
    transport:
      type: streamable-http
      url: https://example.org/mcp
      bearer_token_env: RESEARCH_MCP_TOKEN
    allowed:
      resolve:
        effects: [network-read]
"#;
    let configurations = [
        ApplicationConfiguration::parse(json).expect("JSON MCP configuration"),
        ApplicationConfiguration::parse(jsonc).expect("JSONC MCP configuration"),
        serde_norway::from_str::<ApplicationConfiguration>(yaml).expect("YAML MCP configuration"),
    ];
    configurations.iter().for_each(|configuration| {
        assert!(configuration.mcp.contains_key("research"));
        assert!(configuration.validate().is_ok());
    });
    let duplicate = ApplicationConfiguration::parse(
        r#"{"mcp":{"research":{"transport":{"type":"child-process","command":"server"},"allowed":{"resolve":{"effects":["mutation","mutation"]}}}}}"#,
    )
    .expect("parse duplicate effect configuration");
    assert!(duplicate.validate().is_err());
    assert!(ApplicationConfiguration::parse(
        r#"{"mcp":{"research":{"transport":{"type":"child-process","command":"server","url":"https://example.org"},"allowed":{"resolve":{}}}}}"#,
    )
    .is_err());
}
#[test]
fn test_model_update_preserves_jsonc_comments_and_dry_run() {
    let directory = temp_resolve_dir("sync-acorn-config");
    create_dir_all(&directory).unwrap();
    let path = directory.join("config.jsonc");
    let before = "{\n  // Keep this comment\n  \"models\": [\"acme/existing\"]\n}\n";
    write(&path, before).unwrap();
    let configuration = ApplicationConfiguration::read(path.clone()).unwrap();
    let entries = vec![ModelEntry::Selector("acme/added".to_string())];
    let updated = configuration.with_models(&entries).unwrap();
    updated.write_or_preview(&path, true, true).unwrap();
    assert_eq!(read_file(&path).unwrap(), before);
    updated.write_or_preview(&path, false, true).unwrap();
    let content = read_file(&path).unwrap();
    assert_eq!(
        content,
        "{\n  // Keep this comment\n  \"models\": [\n    \"acme/existing\",\n    \"acme/added\"\n  ]\n}\n"
    );
    let _ = remove_dir_all(directory);
}
#[test]
fn test_oci_registry_profile_and_model_source_parse() {
    let configuration = ApplicationConfiguration::parse(
        r#"{
            "registries": {
                "internal": {
                    "kind": "harbor",
                    "endpoint": "https://harbor.example.org",
                    "credentialEnv": "ACORN_HARBOR_TOKEN"
                }
            },
            "models": [{
                "name": "nemotron-oci",
                "source": {
                    "provider": "oci",
                    "location": "oci://harbor.example.org/ai/models/nemotron:v1",
                    "registry": "internal"
                }
            }]
        }"#,
    )
    .unwrap();
    let profiles = configuration.registry_lookup().unwrap();
    assert_eq!(profiles.get("internal").map(|profile| &profile.kind), Some(&RegistryKind::Harbor));
    assert!(matches!(
        configuration.models.as_deref(),
        Some([ModelEntry::Entry(ModelEntryOptions {
            source: Repository::OCI { registry: Some(name), .. },
            ..
        })]) if name == "internal"
    ));
}
#[test]
fn test_oci_registry_profile_rejects_plaintext_secret_and_insecure_endpoint() {
    assert!(ApplicationConfiguration::parse(r#"{"registries":{"bad":{"endpoint":"https://registry.example.org","password":"secret"}}}"#).is_err());
    let configuration =
        ApplicationConfiguration::parse(r#"{"registries":{"bad":{"endpoint":"http://registry.example.org","credentialEnv":"bad-name"}}}"#).unwrap();
    assert!(configuration.registry_lookup().is_err());
}
#[test]
fn test_operations_complete_message_includes_bucket_name_and_guidance() {
    let message = operations_complete_message(Some("acorn".to_string()), 2, 1);
    assert!(message.contains("Obtained"));
    assert!(message.contains("ACORN"));
    assert!(message.contains(" bucket"));
    assert!(message.contains("data file"));
    assert!(message.contains("image"));
    assert!(message.contains("Do you need to add some images?"));
}
#[test]
fn test_operations_complete_message_uses_url_placeholder_without_name() {
    let message = operations_complete_message(None, 0, 0);
    assert!(message.contains("Obtained"));
    assert!(message.contains("<URL>"));
}
#[test]
fn test_parse_supports_yaml_flow_mapping_when_json_detection_fails() {
    assert!(ApplicationConfiguration::parse("{endpoints: []}").is_ok());
}
#[cfg(feature = "agentic")]
#[test]
fn test_power_automate_gitlab_project_id_parses_numeric_identifiers() {
    let configuration = ApplicationConfiguration::parse(
        r#"{"options":{"powerAutomate":{"callback_destination":"pilot-flow","gitlab_project":"22464","package":"artifacts"}}}"#,
    )
    .expect("parse PowerAutomate options");
    assert_eq!(
        configuration
            .power_automate()
            .expect("PowerAutomate options")
            .gitlab_project_id()
            .expect("numeric project"),
        22464
    );
    let padded = ApplicationConfiguration::parse(
        r#"{"options":{"powerAutomate":{"callback_destination":"pilot-flow","gitlab_project":"  42 ","package":"artifacts"}}}"#,
    )
    .expect("parse padded project");
    assert_eq!(
        padded
            .power_automate()
            .expect("PowerAutomate options")
            .gitlab_project_id()
            .expect("padded project"),
        42
    );
    let named = ApplicationConfiguration::parse(
        r#"{"options":{"powerAutomate":{"callback_destination":"pilot-flow","gitlab_project":"pilot","package":"artifacts"}}}"#,
    )
    .expect("parse named project");
    assert!(named.power_automate().expect("PowerAutomate options").gitlab_project_id().is_err());
}
#[cfg(unix)]
#[test]
fn test_prepare_destination_replaces_parent_symlink_without_following_it() {
    let output = temp_resolve_dir("clobber-symlink-output");
    let external = temp_resolve_dir("clobber-symlink-external");
    create_dir_all(&output).unwrap();
    create_dir_all(&external).unwrap();
    write(external.join("index.json"), "outside").unwrap();
    symlink(&external, output.join("linked")).unwrap();
    let destination = SafePath::new("linked/index.json").unwrap();
    let target = destination.prepare(&output).unwrap();
    assert_eq!(target, output.join("linked/index.json"));
    assert!(output.join("linked").is_dir());
    assert!(!output.join("linked").symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(read_to_string(external.join("index.json")).unwrap(), "outside");
    let _ = remove_dir_all(output);
    let _ = remove_dir_all(external);
}
#[test]
fn test_resolution_failure_reason_reports_fallback_lookup_status() {
    assert_eq!(
        ModelEntry::resolution_failure_reason(&"missing", true, &[]),
        "(missing; no fallback repositories found in the local model database)"
    );
    assert_eq!(ModelEntry::resolution_failure_reason(&"missing", false, &[]), "(missing)");
    assert_eq!(
        ModelEntry::resolution_failure_reason(&"missing", true, &["fallback/model".to_string()]),
        "(missing)"
    );
}
#[test]
fn test_resolve_falls_back_to_default_when_provided_path_missing() {
    let directory = temp_resolve_dir("resolve-fallback");
    create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    let default = directory.join(".acorn.yml");
    write(&default, "{}\n").unwrap();
    let resolved = ApplicationConfiguration::resolve_in(&Some(directory.join("missing.json")), &directory);
    assert_eq!(resolved, Some(default));
    let _ = remove_dir_all(directory);
}
#[test]
fn test_resolve_returns_explicit_existing_path() {
    let directory = temp_resolve_dir("resolve-explicit");
    create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    let provided = directory.join("config.yaml");
    write(&provided, "{}\n").unwrap();
    write(directory.join(".acorn.json"), "{}\n").unwrap();
    assert_eq!(ApplicationConfiguration::resolve(&Some(provided.clone())), Some(provided));
    let _ = remove_dir_all(directory);
}
#[test]
fn test_runner_target_accepts_string_or_object() {
    let alias = serde_json::from_str::<RunnerTarget>(r#""oklahoma""#).unwrap();
    let object = serde_json::from_str::<RunnerTarget>(r#"{"host": "oklahoma"}"#).unwrap();
    assert_eq!(alias, object);
    assert_eq!(alias.host.as_str(), "ssh://oklahoma");
    assert_eq!(serde_json::to_value(alias).unwrap(), serde_json::json!({"host": "ssh://oklahoma"}));
}
#[test]
fn test_runner_target_rejects_invalid_or_unknown_values() {
    let invalid_host = r#"{
        "runners": [{
            "repository": {"provider": "gitlab", "id": 42, "location": "https://gitlab.com/example/project"},
            "type": "project",
            "target": {"host": "http://builder"}
        }]
    }"#;
    let unknown_target = r#"{
        "runners": [{
            "repository": {"provider": "gitlab", "id": 42, "location": "https://gitlab.com/example/project"},
            "type": "project",
            "target": {"host": "builder", "context": "prod"}
        }]
    }"#;
    // `OneOrMany` deserializes as `untagged`, which reports only that no variant matched
    // rather than the specific cause. Assert rejection here; the reason is recovered by the
    // `Validate` pass, per the follow-up item in the PA-11 plan.
    assert!(serde_json::from_str::<ApplicationConfiguration>(invalid_host).is_err());
    assert!(serde_json::from_str::<ApplicationConfiguration>(unknown_target).is_err());
}
#[test]
fn test_runners() {
    let path = fixtures_dir().join("config").join("with_runners.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    insta::assert_snapshot!("runners", format!("{:#?}", config.runners));
}
#[test]
fn test_transfer_item_collect_preserves_or_flattens_paths() {
    let preserved = TransferItem::collect(source_items(&["docs/quest/index.json", "docs/quest/image.png"]), false).unwrap();
    let flattened = TransferItem::collect(source_items(&["docs/quest/index.json", "docs/quest/image.png"]), true).unwrap();
    assert_eq!(
        preserved.iter().map(|item| item.destination.clone().into_path_buf()).collect::<Vec<_>>(),
        vec![PathBuf::from("docs/quest/index.json"), PathBuf::from("docs/quest/image.png")]
    );
    assert_eq!(
        flattened.iter().map(|item| item.destination.clone().into_path_buf()).collect::<Vec<_>>(),
        vec![PathBuf::from("index.json"), PathBuf::from("image.png")]
    );
}
#[test]
fn test_transfer_item_collect_rejects_flattened_filename_collisions() {
    let message = TransferItem::collect(source_items(&["one/index.json", "two/index.json"]), true)
        .unwrap_err()
        .to_string()
        .replace('\\', "/");
    assert!(message.contains("index.json"));
    assert!(message.contains("one/index.json"));
    assert!(message.contains("two/index.json"));
}
#[test]
fn test_transfer_item_collect_rejects_unsafe_destination() {
    let result = BucketSourceItem::new("unsafe".to_string(), "../outside.json", None, None);
    assert!(result.unwrap_err().to_string().contains("unsafe"));
}
#[test]
fn test_with_models_keeps_unique_identifiers() {
    let configuration = ApplicationConfiguration::parse(r#"{"models":["acme/existing","acme/existing"]}"#).unwrap();
    let entries = vec![
        ModelEntry::Selector("acme/existing".to_string()),
        ModelEntry::Selector("acme/added".to_string()),
        ModelEntry::Selector("acme/added".to_string()),
    ];
    let identifiers = configuration
        .with_models(&entries)
        .unwrap()
        .models
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| match entry {
            | ModelEntry::Selector(identifier) => Some(identifier),
            | ModelEntry::Entry(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(identifiers, vec!["acme/existing", "acme/added"]);
}
