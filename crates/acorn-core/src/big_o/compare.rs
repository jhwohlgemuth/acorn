//! Conservative asymptotic comparison of parsed Big-O expressions.
use super::{Exponent, Expression, Term};
use crate::util::math::{Degree, Monomial, Rational};
use crate::ThisOrThat;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::EnumIs;

const MAX_ASSUMPTIONS: usize = 64;
const MAX_TERMS: usize = 256;
/// Failure to compare a pair of Big-O expressions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComparisonError {
    /// The left expression is not valid bounded Big-O notation.
    LeftParse(super::ParseError),
    /// The right expression is not valid bounded Big-O notation.
    RightParse(super::ParseError),
    /// The request contains more explicit assumptions than supported.
    TooManyAssumptions {
        /// Number of supplied assumption entries.
        count: usize,
    },
    /// An explicit assumption names no symbol in either expression.
    UnknownAssumption(String),
}
/// Proven relationship between the growth of two bound expressions.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dominance {
    /// The left expression grows strictly faster.
    Left,
    /// The right expression grows strictly faster.
    Right,
    /// Both expressions have the same asymptotic growth.
    Equivalent,
    /// Independent growth directions prove neither expression dominates.
    Incomparable,
    /// Available assumptions or proof rules do not settle the relationship.
    Undetermined,
}
/// Asymptotic behavior assumed for one named symbol.
#[derive(Clone, Copy, Debug, Deserialize, EnumIs, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SymbolGrowth {
    /// An independent input size tending toward infinity.
    Growing,
    /// A fixed positive value.
    FixedPositive,
    /// A fixed logarithm base greater than one.
    FixedLogBase,
    /// A positive value tending toward zero.
    ShrinkingPositive,
}
/// A parsed expression viewed through its inferred asymptotic growth.
///
/// Each expression is normalized independently: ordinary symbols tend toward positive infinity, and symbols used as named logarithm bases within that expression are fixed and greater than one.
/// A comparison partner never changes these classifications.
/// Unsupported expressions have no ordering, even against themselves; independent growth directions can also be incomparable.
///
/// ### Examples
/// ```
/// use acorn_core::big_o::{Asymptotic, Expression};
///
/// match (Expression::parse("O(n^2)"), Expression::parse("O(n)")) {
///     (Ok(quadratic), Ok(linear)) => {
///         let quadratic = Asymptotic::new(&quadratic);
///         let linear = Asymptotic::new(&linear);
///         assert!(quadratic > linear);
///         assert_eq!(quadratic.dominates(linear), Some(true));
///     }
///     (quadratic, linear) => panic!("hard-coded expressions should parse: {quadratic:?}, {linear:?}"),
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Asymptotic<'expression>(&'expression Expression);
/// Serializable request for deterministic pairwise Big-O comparison.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonInput {
    /// Big-O notation on the left side of the comparison.
    #[schemars(length(max = 256))]
    pub left: String,
    /// Big-O notation on the right side of the comparison.
    #[schemars(length(max = 256))]
    pub right: String,
    /// Optional explicit growth classifications keyed by referenced symbol.
    #[serde(default)]
    #[schemars(extend("maxProperties" = 64))]
    pub assumptions: BTreeMap<String, SymbolGrowth>,
}
/// Serializable result of deterministic pairwise Big-O comparison.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonOutput {
    /// Original left notation, preserved unchanged.
    pub left: String,
    /// Original right notation, preserved unchanged.
    pub right: String,
    /// Proven relationship under the effective assumptions.
    pub dominance: Dominance,
    /// Inferred and explicit assumptions used by the proof engine.
    pub assumptions: BTreeMap<String, SymbolGrowth>,
}
/// Caller-supplied growth assumptions shared by both expressions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GrowthAssumptions(BTreeMap<String, SymbolGrowth>);
impl<'expression> Asymptotic<'expression> {
    /// View an expression using inferred asymptotic-growth conventions.
    pub const fn new(expression: &'expression Expression) -> Self {
        Self(expression)
    }
    /// Return whether this expression dominates another comparable expression.
    ///
    /// `Some(false)` means the other expression dominates or both are equivalent.
    /// `None` means the expressions are incomparable or the available proof rules cannot order them.
    #[must_use]
    pub fn dominates(self, other: Self) -> Option<bool> {
        self.partial_cmp(&other).map(|ordering| ordering == Ordering::Greater)
    }
}
impl PartialEq for Asymptotic<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}
impl PartialOrd for Asymptotic<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left = normalize(self.0.root(), &GrowthAssumptions::inferred(self.0));
        let right = normalize(other.0.root(), &GrowthAssumptions::inferred(other.0));
        left.zip(right).and_then(|(left, right)| match compare_terms(&left, &right) {
            | Dominance::Left => Some(Ordering::Greater),
            | Dominance::Right => Some(Ordering::Less),
            | Dominance::Equivalent => Some(Ordering::Equal),
            | Dominance::Incomparable | Dominance::Undetermined => None,
        })
    }
}
impl core::error::Error for ComparisonError {}
impl fmt::Display for ComparisonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::LeftParse(error) => write!(formatter, "invalid left Big-O expression: {error}"),
            | Self::RightParse(error) => write!(formatter, "invalid right Big-O expression: {error}"),
            | Self::TooManyAssumptions { count } => {
                write!(formatter, "too many growth assumptions: {count} exceeds {MAX_ASSUMPTIONS}")
            }
            | Self::UnknownAssumption(name) => {
                write!(formatter, "growth assumption `{name}` does not reference either expression")
            }
        }
    }
}
impl GrowthAssumptions {
    /// Start with no classified symbols.
    pub fn new() -> Self {
        Self::default()
    }
    /// Infer classifications shared by a pair of parsed expressions.
    pub fn inferred_pair(values: ThisOrThat<&Expression>) -> Self {
        let (left, right) = values.into_parts();
        left.variables()
            .chain(right.variables())
            .fold(Self::new(), |assumptions, name| assumptions.with(name, SymbolGrowth::Growing))
            .infer_fixed_log_bases(left.root())
            .infer_fixed_log_bases(right.root())
    }
    /// Return the classification for one symbol.
    pub fn get(&self, name: &str) -> Option<SymbolGrowth> {
        self.0.get(name).copied()
    }
    /// Iterate over classified symbols in lexical order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, SymbolGrowth)> {
        self.0.iter().map(|(name, growth)| (name.as_str(), *growth))
    }
    /// Classify one symbol, replacing an earlier classification if present.
    pub fn with(mut self, name: impl Into<String>, growth: SymbolGrowth) -> Self {
        self.0.insert(name.into(), growth);
        self
    }
    fn inferred(expression: &Expression) -> Self {
        expression
            .variables()
            .fold(Self::new(), |assumptions, name| assumptions.with(name, SymbolGrowth::Growing))
            .infer_fixed_log_bases(expression.root())
    }
    fn infer_fixed_log_bases(self, term: &Term) -> Self {
        match term {
            | Term::Add(terms) | Term::Multiply(terms) => terms.iter().fold(self, Self::infer_fixed_log_bases),
            | Term::Constant(_) | Term::Variable(_) => self,
            | Term::Divide(numerator, denominator) => self.infer_fixed_log_bases(numerator).infer_fixed_log_bases(denominator),
            | Term::Log(argument) | Term::OutputWork(argument) | Term::Polylog(argument) => self.infer_fixed_log_bases(argument),
            | Term::LogBase { base, argument } => self.with(base, SymbolGrowth::FixedLogBase).infer_fixed_log_bases(argument),
            | Term::Power { base, .. } => self.infer_fixed_log_bases(base),
        }
    }
    fn overlay(self, explicit: &BTreeMap<String, SymbolGrowth>) -> Self {
        explicit
            .iter()
            .fold(self, |assumptions, (name, growth)| assumptions.with(name.clone(), *growth))
    }
}
impl SymbolGrowth {
    fn is_non_growing(self) -> bool {
        self.is_fixed_positive() || self.is_fixed_log_base() || self.is_shrinking_positive()
    }
}
/// Compare the growth of two validated Big-O expressions under explicit symbol assumptions.
///
/// A result other than [`Dominance::Undetermined`] is proved for the supplied assumptions.
#[must_use]
pub fn compare_growth(values: ThisOrThat<&Expression>, assumptions: &GrowthAssumptions) -> Dominance {
    let (left, right) = values.into_parts();
    if left.root() == right.root() && left.variables().all(|name| assumptions.get(name).is_some()) {
        Dominance::Equivalent
    } else {
        match (normalize(left.root(), assumptions), normalize(right.root(), assumptions)) {
            | (Some(left), Some(right)) => compare_terms(&left, &right),
            | _ => Dominance::Undetermined,
        }
    }
}
/// Parse and compare two Big-O expressions under deterministic effective assumptions.
///
/// Original notation is preserved in the successful output.
/// Ordinary symbols are inferred as growing, named logarithm bases are inferred as fixed, and explicit assumptions override those defaults.
pub fn compare_notation(input: ComparisonInput) -> Result<ComparisonOutput, ComparisonError> {
    let ComparisonInput { left, right, assumptions } = input;
    match (Expression::parse(&left), Expression::parse(&right)) {
        | (Err(error), _) => Err(ComparisonError::LeftParse(error)),
        | (_, Err(error)) => Err(ComparisonError::RightParse(error)),
        | (Ok(_), Ok(_)) if assumptions.len() > MAX_ASSUMPTIONS => Err(ComparisonError::TooManyAssumptions { count: assumptions.len() }),
        | (Ok(left_expression), Ok(right_expression)) => {
            let referenced = left_expression.variables().chain(right_expression.variables()).collect::<BTreeSet<_>>();
            match assumptions.keys().find(|name| !referenced.contains(name.as_str())) {
                | Some(name) => Err(ComparisonError::UnknownAssumption(name.clone())),
                | None => {
                    let effective = GrowthAssumptions::inferred_pair(ThisOrThat::new(&left_expression, &right_expression)).overlay(&assumptions);
                    let dominance = compare_growth(ThisOrThat::new(&left_expression, &right_expression), &effective);
                    let assumptions = effective.iter().map(|(name, growth)| (name.to_string(), growth)).collect();
                    Ok(ComparisonOutput {
                        left,
                        right,
                        dominance,
                        assumptions,
                    })
                }
            }
        }
    }
}
fn compare_terms(left: &[Monomial], right: &[Monomial]) -> Dominance {
    let left_at_most = left.iter().all(|term| {
        right
            .iter()
            .any(|candidate| term.direction(candidate).is_some_and(|(_, greater)| !greater))
    });
    let right_at_most = right.iter().all(|term| {
        left.iter()
            .any(|candidate| term.direction(candidate).is_some_and(|(_, greater)| !greater))
    });
    let single_terms = left.len() == 1 && right.len() == 1;
    let opposed_directions = left.first().zip(right.first()).and_then(|(a, b)| a.direction(b)) == Some((true, true));
    match (left_at_most, right_at_most) {
        | (true, true) => Dominance::Equivalent,
        | (true, false) if provably_strict(left, right) => Dominance::Right,
        | (false, true) if provably_strict(right, left) => Dominance::Left,
        | (false, false) if single_terms && opposed_directions => Dominance::Incomparable,
        | _ => Dominance::Undetermined,
    }
}
fn provably_strict(smaller: &[Monomial], larger: &[Monomial]) -> bool {
    let single_terms = smaller.len() == 1 && larger.len() == 1;
    let strict_direction = smaller
        .first()
        .zip(larger.first())
        .and_then(|(a, b)| a.direction(b))
        .is_some_and(|(less, greater)| less && !greater);
    let single_term_strict = single_terms && strict_direction;
    single_term_strict
        || matches!((maximum_total_power(smaller), maximum_total_power(larger)),
            (Some(left), Some(right)) if left.compare(right) == Ordering::Less)
}
fn maximum_total_power(terms: &[Monomial]) -> Option<Rational> {
    terms
        .iter()
        .try_fold(None, |maximum: Option<Rational>, term| {
            term.0
                .values()
                .try_fold(Rational::ZERO, |total, factor| total.add(factor.power))
                .map(|degree| match maximum {
                    | Some(previous) => Some(if previous.compare(degree) == Ordering::Greater {
                        previous
                    } else {
                        degree
                    }),
                    | None => Some(degree),
                })
        })
        .flatten()
}
fn normalize(term: &Term, assumptions: &GrowthAssumptions) -> Option<Vec<Monomial>> {
    match term {
        | Term::Add(terms) => terms.iter().try_fold(Vec::new(), |mut result, term| {
            normalize(term, assumptions).and_then(|more| {
                result.extend(more);
                (result.len() <= MAX_TERMS).then_some(result)
            })
        }),
        | Term::Constant(_) => Some(vec![Monomial::default()]),
        | Term::Divide(numerator, denominator) => normalize(denominator, assumptions)
            .filter(|terms| terms.len() == 1)
            .and_then(|mut terms| terms.pop())
            .and_then(|term| term.power(Rational::NEG_ONE))
            .and_then(|reciprocal| normalize(numerator, assumptions).and_then(|terms| product(terms, vec![reciprocal]))),
        | Term::Log(argument) => normalize_log(argument, assumptions),
        | Term::LogBase { base, argument } => assumptions
            .get(base)
            .is_some_and(|growth| growth.is_fixed_log_base())
            .then(|| normalize_log(argument, assumptions))
            .flatten(),
        | Term::Multiply(terms) => terms.iter().try_fold(vec![Monomial::default()], |acc, term| {
            normalize(term, assumptions).and_then(|right| product(acc, right))
        }),
        | Term::OutputWork(argument) => normalize(argument, assumptions),
        | Term::Polylog(argument) => match argument.as_ref() {
            | Term::Variable(name) if assumptions.get(name).is_some_and(|growth| growth.is_growing()) => {
                let degree = Degree {
                    polylog: 1,
                    ..Degree::default()
                };
                let factor = Monomial::factor(name, degree);
                Some(vec![factor])
            }
            | _ => None,
        },
        | Term::Power { base, exponent } => match exponent {
            | Exponent::Integer(value) => Rational::new(i128::from(*value), 1).and_then(|exponent| normalize_raised(base, exponent, assumptions)),
            | Exponent::Rational(numerator, denominator) => {
                Rational::new(i128::from(*numerator), i128::from(*denominator)).and_then(|exponent| normalize_raised(base, exponent, assumptions))
            }
            | Exponent::Variable(name) => {
                let exponent_is_non_growing = assumptions.get(name).is_some_and(SymbolGrowth::is_non_growing);
                let base_is_constant = normalize(base, assumptions) == Some(vec![Monomial::default()]);
                (exponent_is_non_growing && base_is_constant).then(|| vec![Monomial::default()])
            }
            | Exponent::Difference(left, right) => {
                let left_is_non_growing = assumptions.get(left).is_some_and(SymbolGrowth::is_non_growing);
                let right_is_non_growing = assumptions.get(right).is_some_and(SymbolGrowth::is_non_growing);
                let base_is_constant = normalize(base, assumptions) == Some(vec![Monomial::default()]);
                (left_is_non_growing && right_is_non_growing && base_is_constant).then(|| vec![Monomial::default()])
            }
        },
        | Term::Variable(name) => assumptions.get(name).map(|growth| match growth {
            | SymbolGrowth::Growing => {
                let degree = Degree {
                    power: Rational::ONE,
                    ..Degree::default()
                };
                let factor = Monomial::factor(name, degree);
                vec![factor]
            }
            | SymbolGrowth::FixedPositive | SymbolGrowth::FixedLogBase => vec![Monomial::default()],
            | SymbolGrowth::ShrinkingPositive => {
                let degree = Degree {
                    power: Rational::NEG_ONE,
                    ..Degree::default()
                };
                let factor = Monomial::factor(name, degree);
                vec![factor]
            }
        }),
    }
}
fn normalize_raised(base: &Term, exponent: Rational, assumptions: &GrowthAssumptions) -> Option<Vec<Monomial>> {
    normalize(base, assumptions).and_then(|base| {
        if base.len() == 1 {
            base.into_iter().next().and_then(|term| term.power(exponent)).map(|term| vec![term])
        } else if exponent.denominator == 1 && exponent.numerator <= MAX_TERMS as i128 {
            (0..exponent.numerator).try_fold(vec![Monomial::default()], |acc, _| product(acc, base.clone()))
        } else {
            None
        }
    })
}
fn normalize_log(argument: &Term, assumptions: &GrowthAssumptions) -> Option<Vec<Monomial>> {
    normalize(argument, assumptions)
        .filter(|terms| terms.len() == 1)
        .and_then(|mut terms| terms.pop())
        .and_then(|term| {
            term.0.into_iter().try_fold(Vec::new(), |mut result, (name, degree)| {
                (degree.power.compare(Rational::ZERO) == Ordering::Greater && degree.log == Rational::ZERO && degree.polylog == 0).then(|| {
                    result.push(Monomial::factor(
                        &name,
                        Degree {
                            log: Rational::ONE,
                            ..Degree::default()
                        },
                    ));
                    result
                })
            })
        })
        .filter(|result| !result.is_empty())
}
fn product(left: Vec<Monomial>, right: Vec<Monomial>) -> Option<Vec<Monomial>> {
    left.len().checked_mul(right.len()).filter(|size| *size <= MAX_TERMS).and_then(|_| {
        left.into_iter()
            .flat_map(|a| right.iter().cloned().map(move |b| a.clone().multiply(b)))
            .collect::<Option<Vec<_>>>()
    })
}
