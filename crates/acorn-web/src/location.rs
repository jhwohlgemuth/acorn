//! Source positions for browser results.
use acorn_schema::pid::{Identifier, PID};
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
pub(crate) fn identifier_location(source: &str, kind: &str, normalized: &str) -> Option<(u64, u64, u64, u64)> {
    identifier_locations(source, kind, normalized).into_iter().next()
}
pub(crate) fn identifier_locations(source: &str, kind: &str, normalized: &str) -> Vec<(u64, u64, u64, u64)> {
    let kind = PID::from(kind);
    let characters = source
        .char_indices()
        .filter(|(_, character)| character.is_ascii_alphanumeric())
        .collect::<Vec<_>>();
    let compact_source = characters.iter().map(|(_, character)| character.to_ascii_lowercase()).collect::<String>();
    let path = normalized
        .split_once("://")
        .and_then(|(_, rest)| rest.split_once('/').map(|(_, path)| path));
    let keys = [Some(normalized), path, normalized.rsplit(['/', ':']).next()]
        .into_iter()
        .flatten()
        .map(compact_identifier)
        .filter(|key| key.len() >= 6)
        .collect::<Vec<_>>();
    let mut ranges = keys
        .iter()
        .flat_map(|key| compact_source.match_indices(key).map(|(offset, _)| (offset, key.len())))
        .filter_map(|(offset, length)| {
            characters
                .get(offset)
                .zip(characters.get(offset.saturating_add(length.saturating_sub(1))))
                .map(|((start, _), (last, _))| (*start, last.saturating_add(1)))
        })
        .filter(|(start, end)| {
            let adjacent = |character: char| character.is_ascii_alphanumeric() || character == '_';
            !source[..*start].chars().next_back().is_some_and(adjacent) && !source[*end..].chars().next().is_some_and(adjacent)
        })
        .filter(|(start, end)| {
            source.get(*start..*end).is_some_and(|matched| {
                Identifier {
                    kind: kind.clone(),
                    value: matched.replace(r"\/", "/"),
                }
                .normalized()
                .is_some_and(|identifier| identifier.kind == kind && identifier.value == normalized)
            })
        })
        .collect::<Vec<_>>();
    ranges.sort_unstable_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));
    ranges
        .into_iter()
        .fold(Vec::<(usize, usize)>::new(), |mut found, range| {
            match found.last_mut() {
                | Some(previous) if range.0 < previous.1 => previous.1 = previous.1.max(range.1),
                | _ => found.push(range),
            }
            found
        })
        .into_iter()
        .filter_map(|(start, end)| position(source, start).zip(position(source, end)))
        .filter_map(
            |((line, character), (end_line, end_character))| match [line, character, end_line, end_character].map(u64::try_from) {
                | [Ok(line), Ok(character), Ok(end_line), Ok(end_character)] => Some((line, character, end_line, end_character)),
                | _ => None,
            },
        )
        .collect()
}
fn compact_identifier(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase())
        .collect()
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
