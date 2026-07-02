//! OpenAPI resource import helpers for ACORN API templates.
use super::Resource;
use crate::io::ApiResult;
use crate::prelude::HashSet;
use color_eyre::eyre::eyre;
use convert_case::{Case, Casing};
use itertools::Itertools;
use serde_norway::Value;

//TODO: Whole module needs refinement/refactoring
/// Parse an OpenAPI document and generate ACORN `Resource` entries.
///
/// This first pass imports endpoint interface metadata only (name, method,
/// and URL template) so generated items can be dropped into
/// `application.json` endpoint resources.
pub fn import_resources_from_openapi(spec: &str) -> ApiResult<Vec<Resource>> {
    match serde_norway::from_str::<Value>(spec).map_err(|why| eyre!("Failed to parse OpenAPI YAML: {why}")) {
        | Ok(document) => import_resources_from_value(&document),
        | Err(why) => Err(why),
    }
}
fn import_resources_from_value(document: &Value) -> ApiResult<Vec<Resource>> {
    match document.get("paths").and_then(Value::as_mapping) {
        | Some(paths) => {
            let resources = paths
                .iter()
                .filter_map(|(path_key, path_value)| path_key.as_str().zip(path_value.as_mapping()))
                .flat_map(|(path, operations)| {
                    operations
                        .iter()
                        .filter_map(move |(method_key, operation)| operation_resource_seed(path, method_key.as_str(), operation))
                })
                .map(|(method, path, operation_id)| Resource {
                    name: resource_name(operation_id, method.as_str(), path),
                    method: super::HttpMethod::from(method.as_str()),
                    template: resource_template(path),
                })
                .fold((HashSet::new(), Vec::new()), |(mut names, mut resources), resource| {
                    let name = unique_name(&mut names, &resource.name);
                    resources.push(Resource { name, ..resource });
                    (names, resources)
                })
                .1
                .into_iter()
                .sorted_by(|left, right| left.name.cmp(&right.name))
                .collect::<Vec<Resource>>();
            Ok(resources)
        }
        | None => Err(eyre!("OpenAPI document is missing a valid paths mapping")),
    }
}
fn operation_resource_seed<'a>(path: &'a str, method: Option<&'a str>, operation: &'a Value) -> Option<(String, &'a str, Option<&'a str>)> {
    method
        .map(str::to_ascii_lowercase)
        .filter(|value| matches!(value.as_str(), "delete" | "get" | "patch" | "post" | "put"))
        .map(|value| (value, path, operation_id(operation)))
}
fn operation_id(operation: &Value) -> Option<&str> {
    operation.as_mapping().and_then(|mapping| {
        mapping
            .iter()
            .find_map(|(key, value)| key.as_str().filter(|field| *field == "operationId").and_then(|_| value.as_str()))
    })
}
fn normalize_token(value: &str) -> String {
    let compacted = value
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '-' })
        .collect::<String>();
    compacted
        .split('-')
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<String>>()
        .join("-")
}
fn operation_fallback_name(method: &str, path: &str) -> String {
    let tokens = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                let raw = segment.trim_start_matches('{').trim_end_matches('}');
                let identifier = normalize_token(raw);
                if identifier.is_empty() {
                    String::from("by-identifier")
                } else {
                    format!("by-{identifier}")
                }
            } else {
                normalize_token(segment)
            }
        })
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<String>>();
    let joined = if tokens.is_empty() { String::from("root") } else { tokens.join("-") };
    format!("{joined}-{method}")
}
fn resource_name(operation_id: Option<&str>, method: &str, path: &str) -> String {
    match operation_id {
        | Some(value) if !value.trim().is_empty() => value.to_case(Case::Kebab),
        | _ => operation_fallback_name(method, path),
    }
}
fn resource_template(path: &str) -> String {
    let (output, _, _) = path
        .chars()
        .fold((String::new(), String::new(), false), |(output, key, in_template), character| {
            match (character, in_template) {
                | ('{', false) => (output, String::new(), true),
                | ('}', true) => (format!("{output}{{{{ {} }}}}", template_key(&key)), String::new(), false),
                | (_, true) => (output, format!("{key}{character}"), true),
                | (_, false) => (format!("{output}{character}"), key, false),
            }
        });
    format!("{{{{ base }}}}{output}{{{{ query }}}}")
}
fn template_key(value: &str) -> String {
    let normalized = value
        .chars()
        .map(|item| if item.is_ascii_alphanumeric() || item == '_' { item } else { '_' })
        .collect::<String>();
    match normalized.is_empty() {
        | true => String::from("identifier"),
        | false => normalized,
    }
}
fn unique_name(names: &mut HashSet<String>, seed: &str) -> String {
    core::iter::once(seed.to_string())
        .chain((2_u32..).map(|index| format!("{seed}-{index}")))
        .find(|candidate| names.insert(candidate.clone()))
        .unwrap_or_else(|| seed.to_string())
}
