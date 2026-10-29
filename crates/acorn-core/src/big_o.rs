//! Bounded parsing for portable Big-O notation.
//!
//! The parser recognizes uppercase `O(...)` expressions containing positive integer
//! constants, ASCII identifiers, addition, explicit or adjacent multiplication,
//! grouping, positive integer or identifier exponents, and `log` expressions.
//! It validates notation only; it does not prove a mathematical bound or infer one from measurements.
use crate::prelude::alloc::{vec, Box, String, ToString, Vec};
use alloc::collections::BTreeSet;
use core::fmt;
use core::str::FromStr;

/// Maximum accepted notation length in bytes.
pub const MAX_INPUT_BYTES: usize = 256;
/// Maximum number of nested grouping or `log(...)` levels.
pub const MAX_NESTING_DEPTH: usize = 16;
/// Maximum number of expression and exponent nodes in the parsed tree.
pub const MAX_PARSED_NODES: usize = 64;

/// A positive exponent in a Big-O expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Exponent {
    /// A positive integer exponent.
    Integer(u64),
    /// An exponent named by an ASCII identifier.
    Variable(String),
}
/// Stable category for a Big-O parse failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseErrorKind {
    /// `log identifier` was followed by an exponent without grouping.
    AmbiguousLogExponent,
    /// The expression is empty or contains only whitespace.
    Empty,
    /// The notation does not begin with uppercase `O(`.
    ExpectedBigO,
    /// An expression term was required at this byte.
    ExpectedExpression,
    /// The input exceeds [`MAX_INPUT_BYTES`].
    InputTooLong,
    /// The input contains a non-ASCII character.
    InvalidCharacter,
    /// An exponent was missing, zero, negative, or malformed.
    InvalidExponent,
    /// An integer constant was zero or exceeded `u64`.
    InvalidInteger,
    /// Grouping exceeds [`MAX_NESTING_DEPTH`].
    NestingLimit,
    /// The parsed tree exceeds [`MAX_PARSED_NODES`].
    NodeLimit,
    /// Non-whitespace input follows the closing `O(...)` delimiter.
    TrailingInput,
    /// A closing delimiter was required before the input ended.
    UnexpectedEnd,
    /// A token is not valid at this point in the grammar.
    UnexpectedToken,
}
/// One node in a parsed Big-O expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Term {
    /// A sum, in source order.
    Add(Vec<Term>),
    /// A positive integer constant.
    Constant(u64),
    /// A logarithm of an expression.
    Log(Box<Term>),
    /// A product, in source order.
    Multiply(Vec<Term>),
    /// A base raised to a positive integer or named exponent.
    Power {
        /// Expression being raised.
        base: Box<Term>,
        /// Positive integer or named exponent.
        exponent: Exponent,
    },
    /// An ASCII identifier.
    Variable(String),
}
/// A parsed Big-O expression and the notation from which it was produced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    notation: String,
    root: Term,
    variables: BTreeSet<String>,
}
/// A located Big-O parse failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    kind: ParseErrorKind,
    offset: usize,
}
struct Parser<'input> {
    input: &'input str,
    nodes: usize,
    position: usize,
}
impl Expression {
    /// Parse bounded Big-O notation.
    ///
    /// Whitespace is ignored around tokens. Identifiers begin with an ASCII letter or underscore and continue with ASCII letters, digits, or underscores.
    /// The word `log` is reserved. `log n` is shorthand for one identifier only; write `log(n^2)` or `(log n)^2` when an exponent is intended.
    pub fn parse(notation: &str) -> Result<Self, ParseError> {
        Parser::new(notation).parse()
    }
    /// Return the original notation unchanged.
    pub fn notation(&self) -> &str {
        &self.notation
    }
    /// Return the root of the parsed expression tree.
    pub fn root(&self) -> &Term {
        &self.root
    }
    /// Iterate over referenced identifiers in lexical order.
    pub fn variables(&self) -> impl Iterator<Item = &str> {
        self.variables.iter().map(String::as_str)
    }
}
impl FromStr for Expression {
    type Err = ParseError;

    fn from_str(notation: &str) -> Result<Self, Self::Err> {
        Self::parse(notation)
    }
}
impl core::error::Error for ParseError {}
impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.kind, self.offset)
    }
}
impl ParseError {
    /// Return the stable failure category.
    pub const fn kind(&self) -> ParseErrorKind {
        self.kind
    }
    /// Return the byte offset of the first offending input.
    pub const fn offset(&self) -> usize {
        self.offset
    }
    const fn new(kind: ParseErrorKind, offset: usize) -> Self {
        Self { kind, offset }
    }
}
impl fmt::Display for ParseErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::AmbiguousLogExponent => "ambiguous shorthand log exponent",
            | Self::Empty => "empty Big-O expression",
            | Self::ExpectedBigO => "expected uppercase O(...) notation",
            | Self::ExpectedExpression => "expected expression",
            | Self::InputTooLong => "Big-O expression is too long",
            | Self::InvalidCharacter => "Big-O expression contains a non-ASCII character",
            | Self::InvalidExponent => "invalid exponent",
            | Self::InvalidInteger => "invalid positive integer",
            | Self::NestingLimit => "Big-O expression nesting limit exceeded",
            | Self::NodeLimit => "Big-O expression node limit exceeded",
            | Self::TrailingInput => "trailing input after Big-O expression",
            | Self::UnexpectedEnd => "unexpected end of Big-O expression",
            | Self::UnexpectedToken => "unexpected token in Big-O expression",
        })
    }
}
impl<'input> Parser<'input> {
    const fn new(input: &'input str) -> Self {
        Self {
            input,
            nodes: 0,
            position: 0,
        }
    }
    fn parse(mut self) -> Result<Expression, ParseError> {
        let input_validation = if self.input.len() > MAX_INPUT_BYTES {
            Err(ParseError::new(ParseErrorKind::InputTooLong, MAX_INPUT_BYTES))
        } else if let Some((offset, _)) = self.input.char_indices().find(|(_, character)| !character.is_ascii()) {
            Err(ParseError::new(ParseErrorKind::InvalidCharacter, offset))
        } else {
            self.skip_whitespace();
            if self.position == self.input.len() {
                Err(ParseError::new(ParseErrorKind::Empty, 0))
            } else if !self.consume_byte(b'O') {
                Err(ParseError::new(ParseErrorKind::ExpectedBigO, self.position))
            } else {
                self.skip_whitespace();
                if self.consume_byte(b'(') {
                    Ok(())
                } else {
                    Err(ParseError::new(ParseErrorKind::ExpectedBigO, self.position))
                }
            }
        };
        input_validation.and_then(|()| self.parse_addition(0)).and_then(|root| {
            self.skip_whitespace();
            let closing_delimiter = if self.position == self.input.len() {
                Err(ParseError::new(ParseErrorKind::UnexpectedEnd, self.position))
            } else if !self.consume_byte(b')') {
                Err(ParseError::new(ParseErrorKind::UnexpectedToken, self.position))
            } else {
                self.skip_whitespace();
                if self.position == self.input.len() {
                    Ok(())
                } else {
                    Err(ParseError::new(ParseErrorKind::TrailingInput, self.position))
                }
            };
            closing_delimiter.map(|()| {
                let mut variables = BTreeSet::new();
                collect_variables(&root, &mut variables);
                Expression {
                    notation: self.input.to_string(),
                    root,
                    variables,
                }
            })
        })
    }
    fn parse_addition(&mut self, depth: usize) -> Result<Term, ParseError> {
        self.parse_multiplication(depth).and_then(|first| {
            let mut terms = vec![first];
            let mut parsed = Ok(());
            while parsed.is_ok() {
                self.skip_whitespace();
                if self.consume_byte(b'+') {
                    parsed = self.parse_multiplication(depth).map(|term| terms.push(term));
                } else {
                    break;
                }
            }
            parsed.and_then(|()| {
                if terms.len() == 1 {
                    Ok(terms.remove(0))
                } else {
                    self.node(self.position, Term::Add(terms))
                }
            })
        })
    }
    fn parse_multiplication(&mut self, depth: usize) -> Result<Term, ParseError> {
        self.parse_power(depth).and_then(|first| {
            let mut terms = vec![first];
            let mut parsed = Ok(());
            while parsed.is_ok() {
                self.skip_whitespace();
                let explicit = self.consume_byte(b'*');
                if explicit || self.starts_primary() {
                    parsed = self.parse_power(depth).map(|term| terms.push(term));
                } else {
                    break;
                }
            }
            parsed.and_then(|()| {
                if terms.len() == 1 {
                    Ok(terms.remove(0))
                } else {
                    self.node(self.position, Term::Multiply(terms))
                }
            })
        })
    }
    fn parse_power(&mut self, depth: usize) -> Result<Term, ParseError> {
        self.parse_primary(depth).and_then(|(base, shorthand_log)| {
            self.skip_whitespace();
            let exponent_offset = self.position;
            if !self.consume_byte(b'^') {
                Ok(base)
            } else if shorthand_log {
                Err(ParseError::new(ParseErrorKind::AmbiguousLogExponent, exponent_offset))
            } else {
                self.skip_whitespace();
                let exponent = match self.peek_byte() {
                    | Some(byte) if byte.is_ascii_digit() => self.parse_positive_integer(ParseErrorKind::InvalidExponent).map(Exponent::Integer),
                    | Some(byte) if is_identifier_start(byte) => Ok(Exponent::Variable(self.parse_identifier())),
                    | _ => Err(ParseError::new(ParseErrorKind::InvalidExponent, self.position)),
                };
                exponent.and_then(|exponent| {
                    self.count_node(exponent_offset).and_then(|()| {
                        self.node(
                            exponent_offset,
                            Term::Power {
                                base: Box::new(base),
                                exponent,
                            },
                        )
                    })
                })
            }
        })
    }
    fn parse_primary(&mut self, depth: usize) -> Result<(Term, bool), ParseError> {
        self.skip_whitespace();
        let offset = self.position;
        match self.peek_byte() {
            | Some(b'(') => {
                self.advance(1);
                next_depth(depth, offset)
                    .and_then(|nested_depth| self.parse_addition(nested_depth))
                    .and_then(|expression| {
                        self.skip_whitespace();
                        if self.position == self.input.len() {
                            Err(ParseError::new(ParseErrorKind::UnexpectedEnd, self.position))
                        } else if !self.consume_byte(b')') {
                            Err(ParseError::new(ParseErrorKind::UnexpectedToken, self.position))
                        } else {
                            Ok((expression, false))
                        }
                    })
            }
            | Some(byte) if byte.is_ascii_digit() => self
                .parse_positive_integer(ParseErrorKind::InvalidInteger)
                .and_then(|value| self.node(offset, Term::Constant(value)))
                .map(|term| (term, false)),
            | Some(byte) if is_identifier_start(byte) && self.at_log_keyword() => self.parse_log(depth),
            | Some(byte) if is_identifier_start(byte) => {
                let identifier = self.parse_identifier();
                self.node(offset, Term::Variable(identifier)).map(|term| (term, false))
            }
            | Some(_) => Err(ParseError::new(ParseErrorKind::ExpectedExpression, offset)),
            | None => Err(ParseError::new(ParseErrorKind::UnexpectedEnd, offset)),
        }
    }
    fn parse_log(&mut self, depth: usize) -> Result<(Term, bool), ParseError> {
        let offset = self.position;
        self.advance(3);
        let before_whitespace = self.position;
        self.skip_whitespace();
        if self.consume_byte(b'(') {
            next_depth(depth, offset)
                .and_then(|nested_depth| self.parse_addition(nested_depth))
                .and_then(|argument| {
                    self.skip_whitespace();
                    if self.position == self.input.len() {
                        Err(ParseError::new(ParseErrorKind::UnexpectedEnd, self.position))
                    } else if !self.consume_byte(b')') {
                        Err(ParseError::new(ParseErrorKind::UnexpectedToken, self.position))
                    } else {
                        self.node(offset, Term::Log(Box::new(argument))).map(|term| (term, false))
                    }
                })
        } else if before_whitespace == self.position || !self.peek_byte().is_some_and(is_identifier_start) {
            Err(ParseError::new(ParseErrorKind::ExpectedExpression, self.position))
        } else {
            let identifier_offset = self.position;
            let identifier = self.parse_identifier();
            self.node(identifier_offset, Term::Variable(identifier))
                .and_then(|argument| self.node(offset, Term::Log(Box::new(argument))))
                .map(|term| (term, true))
        }
    }
    fn parse_identifier(&mut self) -> String {
        let start = self.position;
        self.advance(1);
        while self.peek_byte().is_some_and(is_identifier_continue) {
            self.advance(1);
        }
        self.input.get(start..self.position).unwrap_or_default().to_string()
    }
    fn parse_positive_integer(&mut self, kind: ParseErrorKind) -> Result<u64, ParseError> {
        let start = self.position;
        while self.peek_byte().is_some_and(|byte| byte.is_ascii_digit()) {
            self.advance(1);
        }
        self.input
            .get(start..self.position)
            .unwrap_or_default()
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| ParseError::new(kind, start))
    }
    fn node(&mut self, offset: usize, term: Term) -> Result<Term, ParseError> {
        self.count_node(offset).map(|()| term)
    }
    fn count_node(&mut self, offset: usize) -> Result<(), ParseError> {
        if self.nodes == MAX_PARSED_NODES {
            Err(ParseError::new(ParseErrorKind::NodeLimit, offset))
        } else {
            self.nodes = self.nodes.saturating_add(1);
            Ok(())
        }
    }
    fn at_log_keyword(&self) -> bool {
        let keyword_end = self.position.saturating_add(3);
        let is_log = self
            .input
            .as_bytes()
            .get(self.position..)
            .is_some_and(|remaining| remaining.starts_with(b"log"));
        let has_valid_ending = self.input.as_bytes().get(keyword_end).is_none_or(|byte| !is_identifier_continue(*byte));
        is_log && has_valid_ending
    }
    fn advance(&mut self, bytes: usize) {
        self.position = self.position.saturating_add(bytes).min(self.input.len());
    }
    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.advance(1);
            true
        } else {
            false
        }
    }
    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.position).copied()
    }
    fn skip_whitespace(&mut self) {
        while self.peek_byte().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.advance(1);
        }
    }
    fn starts_primary(&self) -> bool {
        self.peek_byte()
            .is_some_and(|byte| byte == b'(' || byte.is_ascii_digit() || is_identifier_start(byte))
    }
}
fn collect_variables(term: &Term, variables: &mut BTreeSet<String>) {
    match term {
        | Term::Add(terms) | Term::Multiply(terms) => terms.iter().for_each(|term| collect_variables(term, variables)),
        | Term::Constant(_) => {}
        | Term::Log(argument) => collect_variables(argument, variables),
        | Term::Power { base, exponent } => {
            collect_variables(base, variables);
            if let Exponent::Variable(variable) = exponent {
                variables.insert(variable.clone());
            }
        }
        | Term::Variable(variable) => {
            variables.insert(variable.clone());
        }
    }
}
const fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}
const fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}
const fn next_depth(depth: usize, offset: usize) -> Result<usize, ParseError> {
    if depth == MAX_NESTING_DEPTH {
        Err(ParseError::new(ParseErrorKind::NestingLimit, offset))
    } else {
        Ok(depth.saturating_add(1))
    }
}
