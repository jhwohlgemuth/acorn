//! Integration coverage for the portable Big-O parser.
use acorn_core::big_o::{Exponent, Expression, ParseErrorKind, Term, MAX_INPUT_BYTES, MAX_NESTING_DEPTH};

#[test]
fn test_accepted_grammar_and_precedence() {
    for notation in [
        "O(1)",
        "O(n)",
        "O(n + m)",
        "O(n*m)",
        "O(n log n)",
        "O(2n)",
        "O((n + 1)(m + 1))",
        "O(n^2)",
        "O(n^k)",
        "O(log n)",
        "O(log(n^2))",
        "O((log n)^2)",
    ] {
        Expression::parse(notation).expect("parse accepted grammar example");
    }
    let expression = Expression::parse("O(n + m*k^2)").expect("parse precedence example");
    assert!(matches!(
        expression.root(),
        Term::Add(terms)
            if matches!(terms.as_slice(), [Term::Variable(n), Term::Multiply(product)]
                if n == "n" && matches!(product.as_slice(), [Term::Variable(m), Term::Power { base, exponent: Exponent::Integer(2) }]
                    if m == "m" && matches!(base.as_ref(), Term::Variable(k) if k == "k")))
    ));
}
#[test]
fn test_invalid_notation_reports_kind_and_offset() {
    for (notation, kind, offset) in [
        ("", ParseErrorKind::Empty, 0),
        ("o(n)", ParseErrorKind::ExpectedBigO, 0),
        ("O()", ParseErrorKind::ExpectedExpression, 2),
        ("O(0)", ParseErrorKind::InvalidInteger, 2),
        ("O(18446744073709551616)", ParseErrorKind::InvalidInteger, 2),
        ("O(n^0)", ParseErrorKind::InvalidExponent, 4),
        ("O(log n^2)", ParseErrorKind::AmbiguousLogExponent, 7),
        ("O(n / m)", ParseErrorKind::UnexpectedToken, 4),
        ("O(n", ParseErrorKind::UnexpectedEnd, 3),
        ("O(n))", ParseErrorKind::TrailingInput, 4),
        ("O(n²)", ParseErrorKind::InvalidCharacter, 3),
    ] {
        let error = Expression::parse(notation).expect_err("reject malformed notation");
        assert_eq!(error.kind(), kind, "notation {notation:?}");
        assert_eq!(error.offset(), offset, "notation {notation:?}");
    }
}
#[test]
fn test_notation_and_variables_are_preserved() {
    let notation = " O(records * log(records) + batches^workers) ";
    let expression = notation.parse::<Expression>().expect("parse variables");
    assert_eq!(expression.notation(), notation);
    assert_eq!(expression.variables().collect::<Vec<_>>(), ["batches", "records", "workers"]);
}
#[test]
fn test_parse_error_displays_kind_and_offset() {
    let error = Expression::parse("O(n^0)").expect_err("reject zero exponent");
    assert_eq!(error.kind().to_string(), "invalid exponent");
    assert_eq!(error.to_string(), "invalid exponent at byte 4");
}
#[test]
fn test_resource_limits_are_enforced() {
    let long = format!("O({})", "n".repeat(MAX_INPUT_BYTES));
    let error = Expression::parse(&long).expect_err("reject long input");
    assert_eq!(error.kind(), ParseErrorKind::InputTooLong);
    assert_eq!(error.offset(), MAX_INPUT_BYTES);

    let nested = format!("O({}n{})", "(".repeat(MAX_NESTING_DEPTH + 1), ")".repeat(MAX_NESTING_DEPTH + 1));
    let error = Expression::parse(&nested).expect_err("reject excessive nesting");
    assert_eq!(error.kind(), ParseErrorKind::NestingLimit);
    assert_eq!(error.offset(), MAX_NESTING_DEPTH + 2);

    let many_nodes = format!("O({})", core::iter::repeat_n("n", 65).collect::<Vec<_>>().join("+"));
    let error = Expression::parse(&many_nodes).expect_err("reject excessive nodes");
    assert_eq!(error.kind(), ParseErrorKind::NodeLimit);
    assert_eq!(error.offset(), 130);
}
