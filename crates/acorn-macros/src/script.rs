use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, LitStr, Token};

#[derive(Clone, Copy, Eq, PartialEq)]
enum Context {
    Code,
    DoubleQuote,
    LineComment,
    BlockComment,
    Regex,
    SingleQuote,
    Template,
}
#[derive(Debug, Eq, PartialEq)]
pub(super) enum Part {
    Binding(String),
    Literal(String),
}
pub(crate) struct FileInput {
    file: LitStr,
    argument: Expr,
}
impl Context {
    fn name(self) -> &'static str {
        match self {
            | Self::BlockComment | Self::LineComment => "comment",
            | Self::Code => "code",
            | Self::DoubleQuote | Self::SingleQuote => "string literal",
            | Self::Regex => "regular expression",
            | Self::Template => "template literal",
        }
    }
}
impl Parse for FileInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<LitStr>().and_then(|file| {
            input
                .parse::<Token![,]>()
                .and_then(|_| input.parse::<Expr>().map(|argument| Self { file, argument }))
        })
    }
}
pub(crate) fn expand_js(input: &LitStr) -> syn::Result<TokenStream> {
    parse_js(&input.value())
        .map_err(|message| syn::Error::new(input.span(), message))
        .and_then(|parts| {
            let preview = parts
                .iter()
                .map(|part| match part {
                    | Part::Binding(_) => "null",
                    | Part::Literal(value) => value.as_str(),
                })
                .collect::<String>();
            validate_javascript(&preview)
                .map_err(|message| syn::Error::new(input.span(), message))
                .map(|()| {
                    parts.into_iter().fold(
                        quote! { ::core::result::Result::<::std::string::String, ::serde_json::Error>::Ok(::std::string::String::new()) },
                        |script, part| match part {
                            | Part::Binding(name) => {
                                let ident = syn::Ident::new(&name, input.span());
                                quote! {
                                    #script.and_then(|mut __script| {
                                        ::serde_json::to_string(&#ident).map(|__binding| {
                                            __script.push_str(&__binding);
                                            __script
                                        })
                                    })
                                }
                            }
                            | Part::Literal(value) => quote! {
                                #script.map(|mut __script| {
                                    __script.push_str(#value);
                                    __script
                                })
                            },
                        },
                    )
                })
        })
}
pub(crate) fn expand_js_file(input: &FileInput) -> syn::Result<TokenStream> {
    let filename = input.file.value();
    let source = if filename.ends_with(".js") && !filename.contains('/') && !filename.contains('\\') && !filename.contains("..") {
        std::env::var("CARGO_MANIFEST_DIR")
            .map_err(|error| syn::Error::new(input.file.span(), error))
            .and_then(|root| {
                let path = std::path::Path::new(&root).join("assets/scripts").join(&filename);
                std::fs::read_to_string(&path)
                    .map_err(|error| syn::Error::new(input.file.span(), format!("js_file!(...): {}: {error}", path.display())))
            })
    } else {
        Err(syn::Error::new(
            input.file.span(),
            "js_file!(...): expected a .js basename under assets/scripts",
        ))
    };
    let file = &input.file;
    let argument = &input.argument;
    source
        .and_then(|source| {
            validate_authored_script(&source)
                .and_then(|()| validate_javascript(&format!("const __acornArgs = null;\n{source}")))
                .map_err(|message| syn::Error::new(input.file.span(), message))
        })
        .map(|()| {
            quote! {
                ::serde_json::to_string(&(#argument)).map(|__argument| {
                    let mut __script = ::std::string::String::from("const __acornArgs = ");
                    __script.push_str(&__argument);
                    __script.push_str(";\n");
                    __script.push_str(::core::include_str!(::core::concat!(::core::env!("CARGO_MANIFEST_DIR"), "/assets/scripts/", #file)));
                    __script
                })
            }
        })
}
pub(super) fn parse_js(script: &str) -> Result<Vec<Part>, String> {
    let chars = script.chars().collect::<Vec<_>>();
    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut context = Context::Code;
    // A slash divides after a complete expression; where a value is expected, it starts a regex.
    let mut expression_can_end = false;
    let mut control_parens = Vec::new();
    let mut expression_braces = Vec::new();
    let mut pending_control_paren = false;
    let mut escaped = false;
    let mut in_regex_class = false;
    let mut index = 0;
    let mut error = None;
    while index < chars.len() {
        let ch = chars[index];
        let next = chars.get(index + 1).copied();
        if ch == '{' {
            if let Some((name, end)) = placeholder(&chars, index) {
                if context != Context::Code {
                    error = Some(format!("js!(...): interpolation inside a {} is forbidden", context.name()));
                    break;
                }
                let joins_identifier = index
                    .checked_sub(1)
                    .and_then(|previous| chars.get(previous))
                    .is_some_and(|ch| *ch == '.' || *ch == '$' || *ch == '_' || ch.is_ascii_alphanumeric())
                    || chars.get(end).is_some_and(|ch| *ch == '$' || *ch == '_' || ch.is_ascii_alphanumeric());
                if joins_identifier {
                    error = Some("js!(...): interpolation must occupy a JavaScript value position".to_string());
                    break;
                }
                if !literal.is_empty() {
                    parts.push(Part::Literal(core::mem::take(&mut literal)));
                }
                parts.push(Part::Binding(name));
                expression_can_end = true;
                index = end;
                continue;
            }
        }
        if context == Context::Code && (ch == '$' || ch == '_' || ch.is_ascii_alphabetic()) {
            let end = chars
                .iter()
                .enumerate()
                .skip(index + 1)
                .find_map(|(position, ch)| (!(ch == &'$' || ch == &'_' || ch.is_ascii_alphanumeric())).then_some(position))
                .unwrap_or(chars.len());
            let word = chars[index..end].iter().collect::<String>();
            let property = index > 0 && chars[index - 1] == '.';
            pending_control_paren = !property && matches!(word.as_str(), "if" | "while" | "for" | "with" | "switch" | "catch");
            expression_can_end = property
                || !matches!(
                    word.as_str(),
                    "return"
                        | "throw"
                        | "case"
                        | "delete"
                        | "void"
                        | "typeof"
                        | "instanceof"
                        | "in"
                        | "of"
                        | "new"
                        | "await"
                        | "yield"
                        | "else"
                        | "do"
                        | "if"
                        | "while"
                        | "for"
                        | "with"
                        | "switch"
                        | "catch"
                );
            literal.push_str(&word);
            index = end;
            continue;
        }
        literal.push(ch);
        match context {
            | Context::Code => match (ch, next) {
                | ('\'', _) => {
                    context = Context::SingleQuote;
                    pending_control_paren = false;
                }
                | ('"', _) => {
                    context = Context::DoubleQuote;
                    pending_control_paren = false;
                }
                | ('`', _) => {
                    context = Context::Template;
                    pending_control_paren = false;
                }
                | ('/', Some('/')) => context = Context::LineComment,
                | ('/', Some('*')) => context = Context::BlockComment,
                | ('/', _) => {
                    if expression_can_end {
                        expression_can_end = false;
                    } else {
                        context = Context::Regex;
                    }
                    pending_control_paren = false;
                }
                | ('(', _) => {
                    control_parens.push(pending_control_paren);
                    pending_control_paren = false;
                    expression_can_end = false;
                }
                | (')', _) => {
                    expression_can_end = !control_parens.pop().unwrap_or(false);
                    pending_control_paren = false;
                }
                | ('{', _) => {
                    // Ambiguous braces are treated as blocks so a following regex stays protected.
                    let previous = chars[..index].iter().rev().find(|ch| !ch.is_whitespace());
                    expression_braces.push(previous.is_some_and(|ch| matches!(ch, '=' | '(' | '[' | ',' | ':')));
                    expression_can_end = false;
                    pending_control_paren = false;
                }
                | ('}', _) => {
                    expression_can_end = expression_braces.pop().unwrap_or(false);
                    pending_control_paren = false;
                }
                | (']', _) => {
                    expression_can_end = true;
                    pending_control_paren = false;
                }
                | ('+', Some('+')) | ('-', Some('-')) => {
                    literal.push(ch);
                    pending_control_paren = false;
                    index += 2;
                    continue;
                }
                | (ch, _) if ch.is_ascii_digit() => {
                    expression_can_end = true;
                    pending_control_paren = false;
                }
                | (ch, _) if ch.is_whitespace() => {}
                | _ => {
                    expression_can_end = false;
                    pending_control_paren = false;
                }
            },
            | Context::SingleQuote | Context::DoubleQuote | Context::Template | Context::Regex => {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if context == Context::Regex {
                    if ch == '[' {
                        in_regex_class = true;
                    } else if ch == ']' {
                        in_regex_class = false;
                    } else if ch == '/' && !in_regex_class {
                        context = Context::Code;
                        expression_can_end = true;
                    }
                } else if (context == Context::SingleQuote && ch == '\'')
                    || (context == Context::DoubleQuote && ch == '"')
                    || (context == Context::Template && ch == '`')
                {
                    context = Context::Code;
                    expression_can_end = true;
                }
            }
            | Context::LineComment if ch == '\n' => context = Context::Code,
            | Context::BlockComment if ch == '/' && index > 0 && chars[index - 1] == '*' => context = Context::Code,
            | _ => {}
        }
        index += 1;
    }
    match error {
        | Some(error) => Err(error),
        | None => {
            if !literal.is_empty() {
                parts.push(Part::Literal(literal));
            }
            Ok(parts)
        }
    }
}
fn placeholder(chars: &[char], start: usize) -> Option<(String, usize)> {
    chars
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, ch)| (*ch == '}').then_some(index))
        .and_then(|end| {
            let name = chars[start + 1..end].iter().collect::<String>();
            let valid = name.chars().next().is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
                && name.chars().all(|ch| ch == '_' || ch.is_ascii_alphanumeric());
            valid.then_some((name, end + 1))
        })
}
pub(super) fn validate_authored_script(source: &str) -> Result<(), String> {
    parse_js(source).and_then(|parts| {
        if parts.iter().any(|part| matches!(part, Part::Binding(_))) {
            Err("js_file!(...): authored scripts must use __acornArgs instead of textual placeholders".into())
        } else {
            Ok(())
        }
    })
}
#[cfg(not(feature = "js-lint"))]
pub(super) fn validate_javascript(_source: &str) -> Result<(), String> {
    Ok(())
}
#[cfg(feature = "js-lint")]
pub(super) fn validate_javascript(source: &str) -> Result<(), String> {
    use oxc_diagnostics::Severity;

    let allocator = oxc_allocator::Allocator::default();
    let wrapped = format!("async function run() {{\n{source}\n}}");
    let parsed = oxc_parser::Parser::new(&allocator, &wrapped, oxc_span::SourceType::default()).parse();
    let minimum = std::env::var("ACORN_SCRIPT_LINT_MIN_SEVERITY").unwrap_or_else(|_| "advice".into());
    let threshold = match minimum.to_ascii_lowercase().as_str() {
        | "error" => 2,
        | "warning" => 1,
        | _ => 0,
    };
    let messages = parsed
        .diagnostics
        .into_iter()
        .filter(|diagnostic| {
            let level = match diagnostic.severity {
                | Severity::Error => 2,
                | Severity::Warning => 1,
                | Severity::Advice => 0,
            };
            level >= threshold
        })
        .map(|diagnostic| diagnostic.message.to_string())
        .collect::<Vec<_>>();
    if messages.is_empty() {
        Ok(())
    } else {
        let shown = messages.iter().take(5).map(String::as_str).collect::<Vec<_>>().join("; ");
        let more = messages.len().saturating_sub(5);
        Err(if more == 0 {
            format!("JavaScript parse failed: {shown}")
        } else {
            format!("JavaScript parse failed: {shown}; and {more} more")
        })
    }
}
