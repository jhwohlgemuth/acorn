use quote::quote;
use syn::LitStr;

#[derive(Debug)]
pub(crate) enum CmdWord {
    Literal(String),
    Splat(String),
    Var(String),
}
pub(crate) fn expand_cmd_sh_words(input: &LitStr) -> syn::Result<proc_macro2::TokenStream> {
    parse_cmd_words(&input.value())
        .map_err(|message| syn::Error::new(input.span(), message))
        .map(|words| {
            let parts = words.into_iter().map(|word| match word {
                | CmdWord::Literal(value) => quote! {
                    ::std::iter::once(::std::ffi::OsString::from(#value)).collect::<::std::vec::Vec<::std::ffi::OsString>>()
                },
                | CmdWord::Splat(name) => {
                    let ident = syn::Ident::new(&name, input.span());
                    quote! {
                        (&#ident).into_iter().map(|__x| {
                            ::std::convert::AsRef::<::std::ffi::OsStr>::as_ref(&__x).to_os_string()
                        }).collect::<::std::vec::Vec<::std::ffi::OsString>>()
                    }
                }
                | CmdWord::Var(name) => {
                    let ident = syn::Ident::new(&name, input.span());
                    quote! {
                        ::std::iter::once(
                            ::std::convert::AsRef::<::std::ffi::OsStr>::as_ref(&#ident).to_os_string()
                        ).collect::<::std::vec::Vec<::std::ffi::OsString>>()
                    }
                }
            });
            quote! {
                [#(#parts),*].concat()
            }
        })
}
pub(crate) fn parse_cmd_words(command: &str) -> Result<Vec<CmdWord>, String> {
    let chars = command.chars().collect::<Vec<_>>();
    let mut words = Vec::new();
    let mut current = String::new();
    let mut index = 0;
    let parsed: Result<(), String> = 'parse: loop {
        if index >= chars.len() {
            break Ok(());
        }
        match chars[index] {
            | ch if ch.is_whitespace() => {
                push_literal(&mut words, &mut current);
                index += 1;
            }
            | '\'' => {
                index += 1;
                while index < chars.len() && chars[index] != '\'' {
                    current.push(chars[index]);
                    index += 1;
                }
                if index == chars.len() {
                    break 'parse Err("cmd!(sh ...): unterminated single quote".to_string());
                }
                index += 1;
            }
            | '"' => {
                index += 1;
                while index < chars.len() && chars[index] != '"' {
                    if chars[index] == '{' && read_interpolation(&chars, index).is_some() {
                        break 'parse Err("cmd!(...): interpolation must occupy an unquoted shell word".to_string());
                    }
                    if chars[index] == '\\' {
                        index += 1;
                        if index == chars.len() {
                            break 'parse Err("cmd!(...): trailing escape in double quote".to_string());
                        }
                    }
                    current.push(chars[index]);
                    index += 1;
                }
                if index == chars.len() {
                    break 'parse Err("cmd!(...): unterminated double quote".to_string());
                }
                index += 1;
            }
            | '\\' => {
                index += 1;
                if index == chars.len() {
                    break 'parse Err("cmd!(...): trailing escape".to_string());
                }
                current.push(chars[index]);
                index += 1;
            }
            | '{' => match read_interpolation(&chars, index) {
                | Some((name, false, next)) if current.is_empty() && next_is_word_boundary(&chars, next) => {
                    words.push(CmdWord::Var(name));
                    index = next;
                }
                | Some((name, true, next)) if current.is_empty() && next_is_word_boundary(&chars, next) => {
                    words.push(CmdWord::Splat(name));
                    index = next;
                }
                | Some(_) => break 'parse Err("cmd!(...): interpolation must occupy a full shell word".to_string()),
                | None => {
                    current.push(chars[index]);
                    index += 1;
                }
            },
            | ch => {
                current.push(ch);
                index += 1;
            }
        }
    };
    parsed.and_then(|()| {
        push_literal(&mut words, &mut current);
        match words.is_empty() {
            | true => Err("cmd!(...): empty command string".to_string()),
            | false => Ok(words),
        }
    })
}
fn is_ident(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic()) && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
fn next_is_word_boundary(chars: &[char], index: usize) -> bool {
    chars.get(index).is_none_or(|ch| ch.is_whitespace())
}
fn push_literal(words: &mut Vec<CmdWord>, current: &mut String) {
    if !current.is_empty() {
        words.push(CmdWord::Literal(core::mem::take(current)));
    }
}
fn read_interpolation(chars: &[char], start: usize) -> Option<(String, bool, usize)> {
    chars
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, ch)| (*ch == '}').then_some(index))
        .and_then(|close| {
            let content = chars[start + 1..close].iter().collect::<String>();
            match content.strip_suffix("...") {
                | Some(name) => is_ident(name).then(|| (name.to_string(), true, close + 1)),
                | None if is_ident(&content) => Some((content, false, close + 1)),
                | None => None,
            }
        })
}
