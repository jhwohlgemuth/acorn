//! Source positions for browser validation diagnostics.
use jsonc_parser::ast::Value as AstValue;
use jsonc_parser::common::{Range, Ranged};
use serde_json::Value;

/// Add one-based source positions to resolvable JSON paths and syntax errors.
pub(crate) fn annotate(value: &mut Value, source: &str, format: &str) {
    if matches!(format, "json" | "jsonc" | "json-ld") {
        let parsed = jsonc_parser::parse_to_ast(source, &Default::default(), &Default::default());
        let root = parsed.as_ref().ok().and_then(|parsed| parsed.value.as_ref());
        let parse_range = parsed.as_ref().err().map(|error| error.range());
        if let Some(diagnostics) = value.get_mut("diagnostics").and_then(Value::as_array_mut) {
            let syntax_range = (diagnostics.len() == 1).then_some(parse_range).flatten();
            for diagnostic in diagnostics {
                let range = diagnostic
                    .get("path")
                    .and_then(Value::as_str)
                    .and_then(|path| root.and_then(|root| source_range(root, path)))
                    .or_else(|| {
                        (diagnostic.get("kind").and_then(Value::as_str) == Some("parse"))
                            .then_some(syntax_range)
                            .flatten()
                    });
                let location = range.and_then(|range| position(source, range.start).map(|start| (start, range)));
                if let (Some(((line, character), range)), Some(object)) = (location, diagnostic.as_object_mut()) {
                    object.insert("line".to_string(), line.into());
                    object.insert("character".to_string(), character.into());
                    if let Some((end_line, end_character)) = (range.end > range.start).then(|| position(source, range.end)).flatten() {
                        object.insert("endLine".to_string(), end_line.into());
                        object.insert("endCharacter".to_string(), end_character.into());
                    }
                }
            }
        }
    }
}
fn position(source: &str, offset: usize) -> Option<(usize, usize)> {
    source.get(..offset).map(|prefix| {
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count().saturating_add(1);
        let character = prefix.rsplit('\n').next().unwrap_or("").chars().count().saturating_add(1);
        (line, character)
    })
}
fn source_range(root: &AstValue<'_>, path: &str) -> Option<Range> {
    let path = path.trim_start_matches('$').trim_start_matches('.');
    (!path.is_empty()).then_some(path).and_then(|path| {
        path.split(['.', '[', ']'])
            .filter(|segment| !segment.is_empty())
            .try_fold(root, |node, segment| match node {
                | AstValue::Object(object) => object.get(segment).map(|property| &property.value),
                | AstValue::Array(array) => segment.parse::<usize>().ok().and_then(|index| array.elements.get(index)),
                | _ => None,
            })
            .map(Ranged::range)
    })
}

#[cfg(test)]
mod tests;
