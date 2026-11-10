//! LaTeX rendering for parsed Big-O expressions.
use super::{Exponent, Expression, Term};
use crate::prelude::alloc::{format, String, ToString, Vec};
use core::fmt;

/// Borrow the string representation of a formatted expression.
pub trait AsStr {
    /// Return the formatted source without surrounding math delimiters.
    fn as_str(&self) -> &str;
}
/// A Big-O expression formatted for LaTeX math mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LatexExpression(String);
impl AsStr for LatexExpression {
    fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<Expression> for LatexExpression {
    fn from(expression: Expression) -> Self {
        Self(format!(r"\mathcal{{O}}\left({}\right)", render_term(&expression.root)))
    }
}
impl fmt::Display for LatexExpression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
fn render_exponent(exponent: &Exponent) -> String {
    match exponent {
        | Exponent::Difference(left, right) => format!("{}-{}", render_identifier(left), render_identifier(right)),
        | Exponent::Integer(value) => value.to_string(),
        | Exponent::Rational(numerator, denominator) => format!(r"\frac{{{numerator}}}{{{denominator}}}"),
        | Exponent::Variable(name) => render_identifier(name),
    }
}
fn render_identifier(name: &str) -> String {
    let escaped = name.replace('_', r"\_");
    if name.len() == 1 {
        escaped
    } else {
        format!(r"\mathrm{{{escaped}}}")
    }
}
fn render_term(term: &Term) -> String {
    match term {
        | Term::Add(terms) => terms.iter().map(render_term).collect::<Vec<_>>().join(" + "),
        | Term::Constant(value) => value.to_string(),
        | Term::Divide(numerator, denominator) => format!(r"\frac{{{}}}{{{}}}", render_term(numerator), render_term(denominator)),
        | Term::Log(argument) => format!(r"\log\left({}\right)", render_term(argument)),
        | Term::LogBase { base, argument } => {
            format!(r"\log_{{{}}}\left({}\right)", render_identifier(base), render_term(argument))
        }
        | Term::Multiply(terms) => terms
            .iter()
            .map(|term| match term {
                | Term::Add(_) => format!(r"\left({}\right)", render_term(term)),
                | _ => render_term(term),
            })
            .collect::<Vec<_>>()
            .join(r" \cdot "),
        | Term::OutputWork(argument) => format!(r"W_{{\mathrm{{out}}}}\left({}\right)", render_term(argument)),
        | Term::Polylog(argument) => format!(r"\operatorname{{polylog}}\left({}\right)", render_term(argument)),
        | Term::Power { base, exponent } => {
            let base = match base.as_ref() {
                | Term::Add(_) | Term::Multiply(_) | Term::Divide(_, _) => format!(r"\left({}\right)", render_term(base)),
                | _ => render_term(base),
            };
            format!("{base}^{{{}}}", render_exponent(exponent))
        }
        | Term::Variable(name) => render_identifier(name),
    }
}
