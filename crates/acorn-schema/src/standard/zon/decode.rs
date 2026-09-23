use super::{validate_encoded, validate_value, Limits, ZonError, MAX_DEPTH};
use super::{
    ZonErrorCode,
    ZonErrorCode::{E001, E002, E003, E004, E101, E102, E103, E104, E303, E304},
};
use crate::validation::rules;
use alloc::{string::String, string::ToString, vec::Vec};
use serde_json::{Map, Number, Value};

struct CsvCell {
    quoted: bool,
    text: String,
}
struct InlineParser<'a> {
    chars: Vec<char>,
    column: usize,
    limits: Limits,
    line: usize,
    position: usize,
    source: &'a str,
}
struct TableHeader {
    columns: Vec<String>,
    rows: usize,
}
impl CsvCell {
    fn parse(line: &str, line_number: usize) -> Result<Vec<Self>, ZonError> {
        let mut cells = Vec::new();
        let mut current = String::new();
        let mut chars = line.chars().peekable();
        let mut quoted = false;
        let mut was_quoted = false;
        while let Some(character) = chars.next() {
            match (character, quoted, chars.peek().copied()) {
                | ('"', true, Some('"')) => {
                    current.push('"');
                    chars.next();
                }
                | ('"', true, _) => quoted = false,
                | ('"', false, _) if current.trim().is_empty() => {
                    current.clear();
                    quoted = true;
                    was_quoted = true;
                }
                | (',', false, _) => {
                    cells.push(Self {
                        quoted: was_quoted,
                        text: if was_quoted { current.clone() } else { current.trim().to_string() },
                    });
                    current.clear();
                    was_quoted = false;
                }
                | (_, _, _) => current.push(character),
            }
        }
        match quoted {
            | true => Err(ZonError::at(E102, line_number, line.chars().count(), line)),
            | false => {
                cells.push(Self {
                    quoted: was_quoted,
                    text: if was_quoted { current } else { current.trim().to_string() },
                });
                Ok(cells)
            }
        }
    }
    fn parse_key(self, line_number: usize, line: &str) -> Result<String, ZonError> {
        match self.quoted {
            | true => unescape(&self.text, line_number, 1, line),
            | false => Ok(self.text),
        }
    }
    fn parse_value(self, line_number: usize, depth: usize, limits: Limits) -> Result<Value, ZonError> {
        match self.quoted {
            | true => unescape(&self.text, line_number, 1, &self.text).map(Value::String),
            | false if self.text.starts_with('\'') => parse_value(&self.text, line_number, 1, depth, limits),
            | false => parse_scalar(&self.text, line_number, 1),
        }
    }
}
impl<'a> InlineParser<'a> {
    fn new(source: &'a str, line: usize, column: usize, limits: Limits) -> Self {
        Self {
            chars: source.chars().collect(),
            column,
            limits,
            line,
            position: 0,
            source,
        }
    }
    fn parse(mut self, depth: usize) -> Result<Value, ZonError> {
        self.value(depth).and_then(|value| {
            self.whitespace();
            match self.position == self.chars.len() {
                | true => Ok(value),
                | false => Err(self.error()),
            }
        })
    }
    fn array(&mut self, depth: usize) -> Result<Value, ZonError> {
        self.position += 1;
        self.whitespace();
        let mut values = Ok(Vec::new());
        while self.peek() != Some(']') && values.is_ok() {
            values = values.and_then(|mut values| match values.len() >= self.limits.array_items {
                | true => Err(ZonError::from(E303)),
                | false => self.value(depth + 1).and_then(|value| {
                    values.push(value);
                    self.whitespace();
                    match self.peek() {
                        | Some(',') => {
                            self.position += 1;
                            self.whitespace();
                            Ok(values)
                        }
                        | Some(']') => Ok(values),
                        | _ => Err(self.error()),
                    }
                }),
            });
        }
        values.map(|values| {
            self.position += 1;
            Value::Array(values)
        })
    }
    fn object(&mut self, depth: usize) -> Result<Value, ZonError> {
        self.position += 1;
        self.whitespace();
        let mut fields = Ok(Map::new());
        while self.peek() != Some('}') && fields.is_ok() {
            fields = fields.and_then(|mut fields| match fields.len() >= self.limits.object_keys {
                | true => Err(ZonError::from(E304)),
                | false => self.key().and_then(|key| {
                    self.whitespace();
                    match self.peek() {
                        | Some(':') => {
                            self.position += 1;
                            self.value(depth + 1)
                        }
                        | Some('{') => self.object(depth + 1),
                        | _ => Err(self.error()),
                    }
                    .and_then(|value| match fields.insert(key, value).is_some() {
                        | true => Err(self.error()),
                        | false => {
                            self.whitespace();
                            match self.peek() {
                                | Some(',') => {
                                    self.position += 1;
                                    self.whitespace();
                                    Ok(fields)
                                }
                                | Some('}') => Ok(fields),
                                | _ => Err(self.error()),
                            }
                        }
                    })
                }),
            });
        }
        fields.map(|fields| {
            self.position += 1;
            Value::Object(fields)
        })
    }
    fn value(&mut self, depth: usize) -> Result<Value, ZonError> {
        match depth > self.limits.depth {
            | true => Err(ZonError::from(E103)),
            | false => {
                self.whitespace();
                match self.peek() {
                    | Some('[') => self.array(depth),
                    | Some('{') => self.object(depth),
                    | Some('"' | '\'') => {
                        let column = self.column + self.position;
                        self.quoted().and_then(|value| parse_nested_quoted_literal(&value, self.line, column))
                    }
                    | Some(_) => {
                        let start = self.position;
                        while self.peek().is_some_and(|value| !matches!(value, ',' | ']' | '}')) {
                            self.position += 1;
                        }
                        parse_scalar(
                            self.chars[start..self.position].iter().collect::<String>().trim(),
                            self.line,
                            self.column + start,
                        )
                    }
                    | None => Err(self.error()),
                }
            }
        }
    }
    fn key(&mut self) -> Result<String, ZonError> {
        match self.peek() {
            | Some('"' | '\'') => self.quoted(),
            | Some(_) => {
                let start = self.position;
                while self.peek().is_some_and(|value| !matches!(value, ':' | '{' | ',' | '}')) {
                    self.position += 1;
                }
                let key = self.chars[start..self.position].iter().collect::<String>().trim().to_string();
                match key.is_empty() {
                    | true => Err(self.coded_error(E104)),
                    | false => Ok(key),
                }
            }
            | None => Err(self.coded_error(E104)),
        }
    }
    fn quoted(&mut self) -> Result<String, ZonError> {
        match self.peek() {
            | None => Err(self.coded_error(E102)),
            | Some(quote) => {
                self.position += 1;
                let mut output = String::new();
                loop {
                    match self.peek() {
                        | None => break Err(self.coded_error(E102)),
                        | Some(character) => {
                            self.position += 1;
                            match character {
                                | value if value == quote => break Ok(output),
                                | '\\' => match self.peek() {
                                    | Some('\\') => output.push('\\'),
                                    | Some('"') => output.push('"'),
                                    | Some('\'') => output.push('\''),
                                    | Some('n') => output.push('\n'),
                                    | Some('r') => output.push('\r'),
                                    | Some('t') => output.push('\t'),
                                    | _ => break Err(self.coded_error(E101)),
                                },
                                | value => output.push(value),
                            }
                            if character == '\\' {
                                self.position += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    fn peek(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }
    fn whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.position += 1;
        }
    }
    fn error(&self) -> ZonError {
        ZonError::at(E103, self.line, self.column + self.position, self.source)
    }
    fn coded_error(&self, code: ZonErrorCode) -> ZonError {
        ZonError::at(code, self.line, self.column + self.position, self.source)
    }
}
impl TableHeader {
    fn parse(line: &str, line_number: usize) -> Result<Self, ZonError> {
        let marker = header_marker(line).map(|(_, marker)| marker).unwrap_or(line);
        marker
            .strip_prefix('@')
            .ok_or_else(|| ZonError::at(E003, line_number, 1, line))
            .and_then(|after_at| match after_at.strip_prefix('(') {
                | Some(parenthesized) => parenthesized.split_once("):").ok_or_else(|| ZonError::at(E003, line_number, 1, line)),
                | None => after_at.split_once(':').ok_or_else(|| ZonError::at(E003, line_number, 1, line)),
            })
            .and_then(|(count, columns)| {
                count
                    .parse::<usize>()
                    .map(|rows| (rows, columns))
                    .map_err(|_| ZonError::at(E003, line_number, 1, line))
            })
            .and_then(|(rows, columns)| {
                CsvCell::parse(columns, line_number).and_then(|columns| {
                    columns
                        .into_iter()
                        .map(|cell| cell.parse_key(line_number, line))
                        .collect::<Result<Vec<_>, _>>()
                        .and_then(|columns| match columns.is_empty() || columns.iter().any(String::is_empty) {
                            | true => Err(ZonError::at(E004, line_number, 1, line)),
                            | false => Ok(Self { columns, rows }),
                        })
                })
            })
    }
}
pub(super) fn decode_value_with_limits(source: &str, limits: Limits) -> Result<Value, ZonError> {
    validate_encoded(source, limits).and_then(|()| {
        let normalized = normalize(source);
        let lines = normalized.lines().filter(|line| !line.is_empty()).collect::<Vec<_>>();
        match lines.as_slice() {
            | [] => Ok(Value::Object(Map::new())),
            | [line] if TableHeader::parse(line, 1).is_err() && !looks_like_object_line(line) => parse_value(line, 1, 1, 0, limits),
            | [first, ..] if looks_like_root_table(first) => {
                parse_table(&lines, 0, 0, limits).and_then(|(value, consumed)| match consumed == lines.len() {
                    | true => Ok(value),
                    | false => Err(ZonError::at(E001, consumed + 1, 1, lines[consumed])),
                })
            }
            | _ => parse_root_object(&lines, limits),
        }
        .and_then(|value| validate_value(&value, 0, limits).map(|()| value))
    })
}
fn header_marker(line: &str) -> Option<(&str, &str)> {
    let mut escaped = false;
    let mut quote = None;
    line.char_indices().find_map(|(index, character)| match (quote, escaped, character) {
        | (Some(_), true, _) => {
            escaped = false;
            None
        }
        | (Some(_), false, '\\') => {
            escaped = true;
            None
        }
        | (Some(mark), false, value) if mark == value => {
            quote = None;
            None
        }
        | (None, _, '"' | '\'') => {
            quote = Some(character);
            None
        }
        | (None, _, '@')
            if (index == 0 || line[..index].ends_with(':'))
                && line[index + 1..]
                    .chars()
                    .next()
                    .is_some_and(|value| value == '(' || value.is_ascii_digit()) =>
        {
            Some((&line[..index], &line[index..]))
        }
        | _ => None,
    })
}
fn insert_unique(fields: &mut Map<String, Value>, key: String, value: Value, line_number: usize, line: &str) -> Result<(), ZonError> {
    match fields.insert(key, value) {
        | None => Ok(()),
        | Some(_) => Err(ZonError::at(E103, line_number, 1, line)),
    }
}
fn keyed_table_header(line: &str, line_number: usize) -> Result<Option<(String, usize, Vec<String>)>, ZonError> {
    match header_marker(line) {
        | Some((key, marker)) if !key.is_empty() => TableHeader::parse(marker, line_number)
            .and_then(|header| parse_key(key.trim_end_matches(':'), line_number, 1).map(|key| Some((key, header.rows, header.columns)))),
        | _ => Ok(None),
    }
}
pub(super) fn looks_like_compound(value: &str) -> bool {
    matches!(
        (value.chars().next(), value.chars().last()),
        (Some('['), Some(']')) | (Some('{'), Some('}'))
    )
}
fn looks_like_object_line(value: &str) -> bool {
    split_object_line(value, 1).is_ok()
}
fn looks_like_root_table(value: &str) -> bool {
    value
        .trim_start()
        .strip_prefix('@')
        .and_then(|rest| rest.chars().next())
        .is_some_and(|character| character == '(' || character.is_ascii_digit())
}
fn normalize(source: &str) -> String {
    source
        .replace("\r\n", "\n")
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
fn parse_compound(source: &str, line: usize, column: usize, depth: usize, limits: Limits) -> Result<Value, ZonError> {
    InlineParser::new(source, line, column, limits).parse(depth)
}
fn parse_key(source: &str, line: usize, column: usize) -> Result<String, ZonError> {
    let source = source.trim();
    match source.chars().next() {
        | Some('"' | '\'') => parse_quoted(source, line, column),
        | _ => Ok(source.to_string()),
    }
}
fn parse_nested_quoted_literal(value: &str, line: usize, column: usize) -> Result<Value, ZonError> {
    match value.chars().next() {
        | Some('"' | '\'') => match parse_quoted(value, line, column) {
            | Ok(inner) if resolves_to_compound(&inner) => Ok(Value::String(inner)),
            | Ok(_) | Err(_) => Ok(Value::String(value.to_string())),
        },
        | _ => Ok(Value::String(value.to_string())),
    }
}
fn parse_object_line(line: &str, line_number: usize, limits: Limits) -> Result<(String, Value), ZonError> {
    split_object_line(line, line_number).and_then(|(key_source, value_source, grouped)| {
        parse_key(key_source, line_number, 1).and_then(|key| match key.is_empty() {
            | true => Err(ZonError::at(E104, line_number, 1, line)),
            | false => {
                let column = key_source.chars().count() + 1;
                match grouped {
                    | true => parse_compound(value_source, line_number, column, 1, limits).map(|value| (key, value)),
                    | false => parse_value(value_source, line_number, column + 1, 1, limits).map(|value| (key, value)),
                }
            }
        })
    })
}
pub(super) fn parse_quoted(source: &str, line: usize, column: usize) -> Result<String, ZonError> {
    source
        .chars()
        .next()
        .ok_or_else(|| ZonError::at(E102, line, column, source))
        .and_then(|quote| match source.len() >= 2 && source.ends_with(quote) {
            | false => Err(ZonError::at(E102, line, column, source)),
            | true => unescape(&source[quote.len_utf8()..source.len() - quote.len_utf8()], line, column, source),
        })
}
fn parse_root_object(lines: &[&str], limits: Limits) -> Result<Value, ZonError> {
    let mut state = Ok((Map::new(), 0usize));
    while state.as_ref().is_ok_and(|(_, index)| *index < lines.len()) {
        state = state.and_then(|(mut fields, index)| {
            let line_number = index + 1;
            let line = lines[index];
            keyed_table_header(line, line_number).and_then(|header| match header {
                | Some((key, _, _)) => parse_table(lines, index, 0, limits)
                    .and_then(|(value, consumed)| insert_unique(&mut fields, key, value, line_number, line).map(|()| (fields, index + consumed))),
                | None => parse_object_line(line, line_number, limits)
                    .and_then(|(key, value)| insert_unique(&mut fields, key, value, line_number, line).map(|()| (fields, index + 1))),
            })
        });
    }
    state.map(|(fields, _)| Value::Object(fields))
}
fn parse_scalar(source: &str, line: usize, column: usize) -> Result<Value, ZonError> {
    let lower = source.to_ascii_lowercase();
    match lower.as_str() {
        | "t" | "true" => Ok(Value::Bool(true)),
        | "f" | "false" => Ok(Value::Bool(false)),
        | "nil" | "none" | "null" => Ok(Value::Null),
        | _ if valid_number(source) => source
            .parse::<Number>()
            .map(Value::Number)
            .map_err(|_| ZonError::at(E103, line, column, source)),
        | _ if rules::numeric(source).is_ok() => Err(ZonError::at(E103, line, column, source)),
        | _ => Ok(Value::String(source.to_string())),
    }
}
fn parse_table(lines: &[&str], start: usize, depth: usize, limits: Limits) -> Result<(Value, usize), ZonError> {
    let line_number = start + 1;
    TableHeader::parse(lines[start], line_number).and_then(|header| {
        let end = start.saturating_add(1).saturating_add(header.rows);
        match end <= lines.len() {
            | false => Err(ZonError::at(E001, line_number, 1, lines[start])),
            | true => lines[start + 1..end]
                .iter()
                .enumerate()
                .map(|(offset, line)| parse_table_row(line, line_number + offset + 1, &header.columns, depth + 1, limits))
                .collect::<Result<Vec<_>, _>>()
                .map(|values| (Value::Array(values), header.rows + 1)),
        }
    })
}
fn parse_table_row(line: &str, line_number: usize, columns: &[String], depth: usize, limits: Limits) -> Result<Value, ZonError> {
    CsvCell::parse(line, line_number).and_then(|cells| match cells.len() == columns.len() {
        | false => Err(ZonError::at(E002, line_number, 1, line)),
        | true => columns
            .iter()
            .zip(cells)
            .map(|(column, cell)| cell.parse_value(line_number, depth, limits).map(|value| (column.clone(), value)))
            .collect::<Result<Map<_, _>, _>>()
            .map(Value::Object),
    })
}
fn parse_value(source: &str, line: usize, column: usize, depth: usize, limits: Limits) -> Result<Value, ZonError> {
    let source = source.trim();
    match source.chars().next() {
        | Some('"' | '\'') => parse_quoted(source, line, column).and_then(|value| match looks_like_compound(&value) {
            | true => parse_compound(&value, line, column, depth + 1, limits),
            | false => parse_nested_quoted_literal(&value, line, column),
        }),
        | Some('[' | '{') => parse_compound(source, line, column, depth + 1, limits),
        | _ => parse_scalar(source, line, column),
    }
}
pub(super) fn resolves_to_compound(value: &str) -> bool {
    core::iter::successors(Some(value.to_string()), |current| match current.chars().next() {
        | Some('"' | '\'') => parse_quoted(current, 1, 1).ok().filter(|inner| inner != current),
        | _ => None,
    })
    .take(MAX_DEPTH.saturating_add(1))
    .any(|current| looks_like_compound(&current))
}
fn split_object_line(line: &str, line_number: usize) -> Result<(&str, &str, bool), ZonError> {
    let mut quote = None;
    let mut escaped = false;
    line.char_indices()
        .find_map(|(index, character)| match (quote, escaped, character) {
            | (Some(_), true, _) => {
                escaped = false;
                None
            }
            | (Some(_), false, '\\') => {
                escaped = true;
                None
            }
            | (Some(mark), false, value) if mark == value => {
                quote = None;
                None
            }
            | (None, _, '"' | '\'') => {
                quote = Some(character);
                None
            }
            | (None, _, ':') => Some((&line[..index], &line[index + 1..], false)),
            | (None, _, '{') if line.ends_with('}') => Some((&line[..index], &line[index..], true)),
            | _ => None,
        })
        .ok_or_else(|| ZonError::at(E103, line_number, 1, line))
}
fn unescape(source: &str, line: usize, column: usize, context: &str) -> Result<String, ZonError> {
    let mut chars = source.chars();
    let mut result = Ok(String::new());
    while result.is_ok() && !chars.as_str().is_empty() {
        result = result.and_then(|mut output| match chars.next() {
            | Some('\\') => match chars.next() {
                | Some('\\') => Ok('\\'),
                | Some('"') => Ok('"'),
                | Some('\'') => Ok('\''),
                | Some('n') => Ok('\n'),
                | Some('r') => Ok('\r'),
                | Some('t') => Ok('\t'),
                | Some(_) => Err(ZonError::at(E101, line, column, context)),
                | None => Err(ZonError::at(E102, line, column, context)),
            }
            .map(|value| {
                output.push(value);
                output
            }),
            | Some(value) => {
                output.push(value);
                Ok(output)
            }
            | None => Ok(output),
        });
    }
    result
}
fn valid_number(source: &str) -> bool {
    match source.is_empty() || source.starts_with('+') {
        | true => false,
        | false => {
            let unsigned = source.strip_prefix('-').unwrap_or(source);
            let leading_zero = unsigned.len() > 1 && unsigned.starts_with('0') && !unsigned.starts_with("0.");
            let digit_after_zero =
                unsigned.starts_with('0') && unsigned.get(1..2).is_some_and(|value| value.chars().all(|value| value.is_ascii_digit()));
            match leading_zero || digit_after_zero {
                | true => false,
                | false => {
                    let lower = unsigned.to_ascii_lowercase();
                    let (base, exponent) = lower
                        .split_once('e')
                        .map_or((lower.as_str(), None), |(base, exponent)| (base, Some(exponent)));
                    let base_valid = match base.split_once('.') {
                        | Some((whole, fraction)) => {
                            !whole.is_empty()
                                && !fraction.is_empty()
                                && whole.chars().all(|value| value.is_ascii_digit())
                                && fraction.chars().all(|value| value.is_ascii_digit())
                        }
                        | None => !base.is_empty() && base.chars().all(|value| value.is_ascii_digit()),
                    };
                    base_valid
                        && exponent.is_none_or(|exponent| {
                            let digits = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
                            !digits.is_empty() && digits.chars().all(|value| value.is_ascii_digit())
                        })
                }
            }
        }
    }
}
