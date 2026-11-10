//! Integration coverage for the portable Big-O parser.
extern crate alloc;

use acorn_core::big_o::{
    compare_growth, compare_notation, AsStr, Asymptotic, ComparisonError, ComparisonInput, Dominance, Exponent, Expression, GrowthAssumptions,
    LatexExpression, ParseErrorKind, SymbolGrowth, Term, MAX_INPUT_BYTES, MAX_NESTING_DEPTH,
};
use acorn_core::ThisOrThat;
use alloc::collections::BTreeMap;
use core::cmp::Ordering;

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
        "O(P^(1/2)+W_out(P))",
        "O(polylog (P) + W_out (P))",
        "O(P*B+2*log(B)*t*H^2/e)",
        "O(G+log_a(G)*k*c^(e-d))",
        "O(T*k*polylog(q/e)*log(1/e))",
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
fn test_latex_expression_formats_big_o_terms() {
    for (notation, expected) in [
        ("O(n^2 + m)", r"\mathcal{O}\left(n^{2} + m\right)"),
        (
            "O((n + 1)(m + 1))",
            r"\mathcal{O}\left(\left(n + 1\right) \cdot \left(m + 1\right)\right)",
        ),
        ("O((log n)^2)", r"\mathcal{O}\left(\log\left(n\right)^{2}\right)"),
        (
            "O(P^(1/2)+W_out(P))",
            r"\mathcal{O}\left(P^{\frac{1}{2}} + W_{\mathrm{out}}\left(P\right)\right)",
        ),
        (
            "O(G+log_a(G)*k*c^(e-d))",
            r"\mathcal{O}\left(G + \log_{a}\left(G\right) \cdot k \cdot c^{e-d}\right)",
        ),
        (
            "O(polylog(q/e) + long_name)",
            r"\mathcal{O}\left(\operatorname{polylog}\left(\frac{q}{e}\right) + \mathrm{long\_name}\right)",
        ),
    ] {
        let expression = Expression::parse(notation).expect("parse LaTeX example");
        let latex = LatexExpression::from(expression);
        assert_eq!(latex.as_str(), expected, "input {notation:?}");
        assert_eq!(latex.to_string(), expected, "input {notation:?}");
    }
}
#[test]
fn test_asymptotic_comparison_infers_standard_growth_conventions() {
    let linear = Expression::parse("O(n)").expect("linear expression");
    let logarithmic = Expression::parse("O(log_a(n))").expect("named logarithm");
    let quadratic = Expression::parse("O(n^2)").expect("quadratic expression");
    let linear = Asymptotic::new(&linear);
    let logarithmic = Asymptotic::new(&logarithmic);
    let quadratic = Asymptotic::new(&quadratic);
    assert!(quadratic > linear);
    assert!(linear > logarithmic);
    assert_eq!(quadratic.dominates(linear), Some(true));
    assert_eq!(linear.dominates(quadratic), Some(false));
    assert_eq!(quadratic.partial_cmp(&linear), Some(Ordering::Greater));
}
#[test]
fn test_asymptotic_comparison_preserves_equivalence_and_incomparability() {
    let linear = Expression::parse("O(n)").expect("linear expression");
    let scaled = Expression::parse("O(2*n+1)").expect("scaled linear expression");
    let independent = Expression::parse("O(m)").expect("independent expression");
    let symbolic = Expression::parse("O(n^m)").expect("symbolic exponent expression");
    assert_ne!(linear, scaled);
    let linear = Asymptotic::new(&linear);
    let scaled = Asymptotic::new(&scaled);
    let independent = Asymptotic::new(&independent);
    let symbolic = Asymptotic::new(&symbolic);
    assert_eq!(linear, scaled);
    assert_eq!(linear.dominates(scaled), Some(false));
    assert_eq!(linear.partial_cmp(&independent), None);
    assert_eq!(symbolic.partial_cmp(&linear), None);
}
#[test]
fn test_asymptotic_equality_is_transitive_for_large_rational_exponents() {
    let expressions = [
        "O(n^(18446744073709551615/18446744073709551615))",
        "O(n)",
        "O(n^(18446744073709551614/18446744073709551614))",
    ]
    .map(|notation| Expression::parse(notation).expect("linear expression"));
    for left in &expressions {
        for right in &expressions {
            assert_eq!(Asymptotic::new(left).partial_cmp(&Asymptotic::new(right)), Some(Ordering::Equal));
        }
    }
}
#[test]
fn test_asymptotic_equality_laws_across_equivalent_forms() {
    let expressions = [
        "O(1)",
        "O(a)",
        "O(log_a(n)/log_a(n))",
        "O(a*log_a(n))",
        "O(log(n))",
        "O(n)",
        "O(2*n+1)",
        "O(n^(2/2))",
        "O(n+n)",
        "O(n+m)",
        "O(m+n)",
        "O(n*m)",
        "O(n^2)",
        "O(n^m)",
        "O(polylog(n))",
        "O(log(n)^2)",
    ]
    .map(|notation| Expression::parse(notation).expect("valid comparison input"));
    let orders: Vec<Vec<_>> = expressions
        .iter()
        .map(|left| {
            expressions
                .iter()
                .map(|right| Asymptotic::new(left).partial_cmp(&Asymptotic::new(right)))
                .collect()
        })
        .collect();
    let order = |i, j| {
        *orders
            .get(i)
            .and_then(|row: &Vec<_>| row.get(j))
            .expect("comparison matrix contains every expression pair")
    };
    for (i, left) in expressions.iter().enumerate() {
        for (j, middle) in expressions.iter().enumerate() {
            assert_eq!(
                order(i, j),
                order(j, i).map(Ordering::reverse),
                "{} vs {}",
                left.notation(),
                middle.notation()
            );
            for (k, right) in expressions.iter().enumerate() {
                if order(i, j) == Some(Ordering::Equal) && order(j, k) == Some(Ordering::Equal) {
                    assert_eq!(
                        order(i, k),
                        Some(Ordering::Equal),
                        "{} == {} == {}",
                        left.notation(),
                        middle.notation(),
                        right.notation()
                    );
                }
                if order(i, j) == Some(Ordering::Less) && order(j, k) == Some(Ordering::Less) {
                    assert_eq!(
                        order(i, k),
                        Some(Ordering::Less),
                        "{} < {} < {}",
                        left.notation(),
                        middle.notation(),
                        right.notation()
                    );
                }
            }
        }
    }
}
#[test]
fn test_asymptotic_symbol_roles_do_not_depend_on_comparison_partners() {
    let growing = Expression::parse("O(a)").expect("growing symbol");
    let quotient = Expression::parse("O(log_a(n)/log_a(n))").expect("constant quotient");
    let constant = Expression::parse("O(1)").expect("constant");
    let a = Asymptotic::new(&growing);
    let b = Asymptotic::new(&quotient);
    let c = Asymptotic::new(&constant);
    assert_eq!(b, c);
    assert!(a > b);
    assert!(a > c);
    assert_eq!(b.partial_cmp(&a), Some(Ordering::Less));
}
#[test]
fn test_compare_notation_infers_and_overrides_assumptions() {
    let inferred = compare_notation(ComparisonInput {
        left: "O(log_a(n))".to_string(),
        right: "O(n)".to_string(),
        assumptions: BTreeMap::new(),
    })
    .expect("infer ordinary symbols and named logarithm bases");
    assert_eq!(inferred.dominance, Dominance::Right);
    assert_eq!(
        inferred.assumptions,
        BTreeMap::from([("a".to_string(), SymbolGrowth::FixedLogBase), ("n".to_string(), SymbolGrowth::Growing)])
    );
    let overridden = compare_notation(ComparisonInput {
        left: "O(n)".to_string(),
        right: "O(1)".to_string(),
        assumptions: BTreeMap::from([("n".to_string(), SymbolGrowth::FixedPositive)]),
    })
    .expect("explicit assumptions override inference");
    assert_eq!(overridden.dominance, Dominance::Equivalent);
    assert_eq!(overridden.assumptions.get("n"), Some(&SymbolGrowth::FixedPositive));
}
#[test]
fn test_compare_notation_preserves_contract_and_serialization() {
    let output = compare_notation(ComparisonInput {
        left: " O(n^2) ".to_string(),
        right: "O(n log n)".to_string(),
        assumptions: BTreeMap::new(),
    })
    .expect("compare canonical fixture expressions");
    assert_eq!(output.left, " O(n^2) ");
    assert_eq!(output.right, "O(n log n)");
    assert_eq!(output.dominance, Dominance::Left);
    assert_eq!(
        serde_json::to_string(&output).expect("serialize comparison output"),
        r#"{"left":" O(n^2) ","right":"O(n log n)","dominance":"left","assumptions":{"n":"growing"}}"#
    );
    let schema = serde_json::to_value(schemars::schema_for!(ComparisonInput)).expect("serialize comparison input schema");
    assert_eq!(schema.pointer("/properties/left/maxLength"), Some(&serde_json::json!(256)));
    assert_eq!(schema.pointer("/properties/right/maxLength"), Some(&serde_json::json!(256)));
    assert_eq!(schema.pointer("/properties/assumptions/maxProperties"), Some(&serde_json::json!(64)));
    assert_eq!(schema.pointer("/additionalProperties"), Some(&serde_json::json!(false)));
}
#[test]
fn test_compare_notation_rejects_invalid_inputs_and_limits() {
    let invalid_left = compare_notation(ComparisonInput {
        left: "O(n - m)".to_string(),
        right: "O(n)".to_string(),
        assumptions: BTreeMap::new(),
    })
    .expect_err("reject invalid left notation");
    assert!(matches!(&invalid_left, ComparisonError::LeftParse(error) if error.offset() == 4));
    assert_eq!(
        invalid_left.to_string(),
        "invalid left Big-O expression: unexpected token in Big-O expression at byte 4"
    );
    let invalid_right = compare_notation(ComparisonInput {
        left: "O(n)".to_string(),
        right: "O(n - m)".to_string(),
        assumptions: BTreeMap::new(),
    })
    .expect_err("reject invalid right notation");
    assert!(matches!(invalid_right, ComparisonError::RightParse(error) if error.offset() == 4));
    let unknown = compare_notation(ComparisonInput {
        left: "O(n)".to_string(),
        right: "O(n^2)".to_string(),
        assumptions: BTreeMap::from([("z".to_string(), SymbolGrowth::Growing), ("a".to_string(), SymbolGrowth::FixedPositive)]),
    })
    .expect_err("reject assumptions for unreferenced symbols");
    assert_eq!(unknown, ComparisonError::UnknownAssumption("a".to_string()));
    let assumptions = (0..64)
        .map(|index| (format!("symbol_{index:02}"), SymbolGrowth::Growing))
        .collect::<BTreeMap<_, _>>();
    let at_limit = compare_notation(ComparisonInput {
        left: "O(n)".to_string(),
        right: "O(n^2)".to_string(),
        assumptions,
    })
    .expect_err("validate assumptions after accepting the count boundary");
    assert!(matches!(at_limit, ComparisonError::UnknownAssumption(_)));
    let assumptions = (0..65)
        .map(|index| (format!("symbol_{index:02}"), SymbolGrowth::Growing))
        .collect::<BTreeMap<_, _>>();
    let over_limit = compare_notation(ComparisonInput {
        left: "O(n)".to_string(),
        right: "O(n^2)".to_string(),
        assumptions,
    })
    .expect_err("reject too many assumptions before symbol validation");
    assert_eq!(over_limit, ComparisonError::TooManyAssumptions { count: 65 });
}
#[test]
fn test_compare_notation_returns_every_dominance() {
    for (left, right, dominance) in [
        ("O(n^2)", "O(n log n)", Dominance::Left),
        ("O(n log n)", "O(n^2)", Dominance::Right),
        ("O(2*n+1)", "O(n)", Dominance::Equivalent),
        ("O(n)", "O(m)", Dominance::Incomparable),
        ("O(polylog(n))", "O(log(n)^2)", Dominance::Undetermined),
    ] {
        let output = compare_notation(ComparisonInput {
            left: left.to_string(),
            right: right.to_string(),
            assumptions: BTreeMap::new(),
        })
        .expect("valid comparison is a successful analytical outcome");
        assert_eq!(output.dominance, dominance, "{left} versus {right}");
    }
}
#[test]
fn test_function_arguments_distinguish_unexpected_tokens_from_end() {
    for function in ["log", "log_a", "polylog", "W_out"] {
        for whitespace in ["", " \t"] {
            let prefix = format!("O({function}(n{whitespace}");
            for suffix in ["-m))", ",m))", "])"] {
                let notation = format!("{prefix}{suffix}");
                let error = Expression::parse(&notation).expect_err("unexpected function argument token");
                assert_eq!(error.kind(), ParseErrorKind::UnexpectedToken, "{notation}");
                assert_eq!(error.offset(), prefix.len(), "{notation}");
            }
            let error = Expression::parse(&prefix).expect_err("unfinished function argument");
            assert_eq!(error.kind(), ParseErrorKind::UnexpectedEnd, "{prefix}");
            assert_eq!(error.offset(), prefix.len(), "{prefix}");
        }
    }
}
#[test]
fn test_growth_comparison_proves_supported_relationships() {
    let assumptions = GrowthAssumptions::new()
        .with("n", SymbolGrowth::Growing)
        .with("m", SymbolGrowth::Growing)
        .with("e", SymbolGrowth::ShrinkingPositive)
        .with("a", SymbolGrowth::FixedLogBase);
    let compare = |left, right| {
        let left = Expression::parse(left).expect("left expression");
        let right = Expression::parse(right).expect("right expression");
        compare_growth(ThisOrThat::new(&left, &right), &assumptions)
    };
    assert_eq!(compare("O(n^2)", "O(n*log(n))"), Dominance::Left);
    assert_eq!(compare("O(n*log(n))", "O(n^2)"), Dominance::Right);
    assert_eq!(compare("O(2*n+1)", "O(n)"), Dominance::Equivalent);
    assert_eq!(compare("O(n)", "O(m)"), Dominance::Incomparable);
    assert_eq!(compare("O(n+m)", "O(n*m)"), Dominance::Right);
    assert_eq!(compare("O(n^(1/2))", "O(n)"), Dominance::Right);
    assert_eq!(compare("O(n/e)", "O(n)"), Dominance::Left);
    assert_eq!(compare("O(W_out(n))", "O(n)"), Dominance::Equivalent);
    assert_eq!(compare("O(log_a(n))", "O(log(n))"), Dominance::Equivalent);
    assert_eq!(compare("O(n+polylog(n))", "O(n)"), Dominance::Equivalent);
    assert_eq!(compare("O(polylog(n))", "O(log(n)^2)"), Dominance::Undetermined);
    assert_eq!(compare("O(n^(m))", "O(n^2)"), Dominance::Undetermined);
}
#[test]
fn test_growth_comparison_requires_symbol_assumptions() {
    let left = Expression::parse("O(n)").expect("left");
    let right = Expression::parse("O(n)").expect("right");
    assert_eq!(
        compare_growth(ThisOrThat::new(&left, &right), &GrowthAssumptions::new()),
        Dominance::Undetermined
    );
}
#[test]
fn test_growth_comparison_returns_undetermined_at_proof_limits() {
    let assumptions = GrowthAssumptions::new()
        .with("e", SymbolGrowth::ShrinkingPositive)
        .with("m", SymbolGrowth::Growing)
        .with("n", SymbolGrowth::Growing)
        .with("q", SymbolGrowth::Growing);
    let right = Expression::parse("O(n)").expect("right expression");
    for notation in ["O((n+m)^9)", "O((n^18446744073709551615)^18446744073709551615)", "O(polylog(q/e))"] {
        let left = Expression::parse(notation).expect("valid notation");
        assert_eq!(
            compare_growth(ThisOrThat::new(&left, &right), &assumptions),
            Dominance::Undetermined,
            "{notation}"
        );
    }
}
#[test]
fn test_growth_comparison_handles_canonical_corpus_pairs() {
    let assumptions = GrowthAssumptions::new()
        .with("B", SymbolGrowth::Growing)
        .with("G", SymbolGrowth::Growing)
        .with("H", SymbolGrowth::Growing)
        .with("P", SymbolGrowth::Growing)
        .with("a", SymbolGrowth::FixedLogBase)
        .with("c", SymbolGrowth::FixedPositive)
        .with("d", SymbolGrowth::FixedPositive)
        .with("R_s", SymbolGrowth::FixedPositive)
        .with("G_q", SymbolGrowth::FixedPositive)
        .with("e", SymbolGrowth::ShrinkingPositive)
        .with("t", SymbolGrowth::FixedPositive);
    let compare = |left, right| {
        let left = Expression::parse(left).expect("left corpus expression");
        let right = Expression::parse(right).expect("right corpus expression");
        compare_growth(ThisOrThat::new(&left, &right), &assumptions)
    };
    assert_eq!(compare("O(P*B)", "O(P*B^2+B^3)"), Dominance::Right);
    assert_eq!(compare("O(P+log(P)^2+P^(1/2))", "O(P)"), Dominance::Equivalent);
    assert_eq!(compare("O(G+polylog(G))", "O(G)"), Dominance::Equivalent);
    assert_eq!(compare("O(P*B+2*log(B)*t*H^2/e)", "O(P*B+log(B)*t*H^2/e)"), Dominance::Equivalent);
    assert_eq!(compare("O(G+G*R_s*G_q+W_out(G))", "O(G)"), Dominance::Equivalent);
    assert_eq!(compare("O(G+log_a(G)*c^(e-d))", "O(G)"), Dominance::Equivalent);
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
        ("O(n^(1/0))", ParseErrorKind::InvalidExponent, 7),
        ("O(log n^2)", ParseErrorKind::AmbiguousLogExponent, 7),
        ("O(n - m)", ParseErrorKind::UnexpectedToken, 4),
        ("O(n*/m)", ParseErrorKind::ExpectedExpression, 4),
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
fn test_named_log_bases_require_identifier_names() {
    for notation in ["O(log_(n))", "O(log_0(n))", "O(log_1(n))", "O(log_2(n))", "O(log_2n(n))", "O(log_1 (n))"] {
        let error = Expression::parse(notation).expect_err("named base must start with a letter or underscore");
        assert_eq!(error.kind(), ParseErrorKind::ExpectedExpression, "{notation}");
        assert_eq!(error.offset(), 6, "{notation}");
    }
    for base in ["a", "a2", "_a", "log_a"] {
        let notation = format!("O(log_{base}(n))");
        let expression = Expression::parse(&notation).expect("valid named base");
        assert!(
            matches!(expression.root(), Term::LogBase { base: parsed, .. } if parsed == base),
            "{notation}"
        );
        assert!(expression.variables().any(|name| name == base), "{notation}");
    }
    for name in ["log_", "log_0", "log_1", "log_2n"] {
        let notation = format!("O({name})");
        let expression = Expression::parse(&notation).expect("ordinary identifier without a function call");
        assert!(matches!(expression.root(), Term::Variable(parsed) if parsed == name), "{notation}");
    }
}
#[test]
fn test_named_log_identifiers_and_bases_are_preserved() {
    for notation in ["O(log_count)", "O(n*log_count)", "O(log_count^2)", "O(log_count + n)"] {
        let expression = Expression::parse(notation).expect("ordinary log-prefixed variable");
        assert!(expression.variables().any(|name| name == "log_count"), "{notation}");
    }
    for notation in ["O(log_log_a(n))", "O(log_log_a (n))", "O(log_log_a\t(n))"] {
        let expression = Expression::parse(notation).expect("named logarithm");
        assert!(matches!(expression.root(), Term::LogBase { base, .. } if base == "log_a"), "{notation}");
        assert_eq!(expression.variables().collect::<Vec<_>>(), ["log_a", "n"], "{notation}");
    }
}
#[test]
fn test_named_log_requires_assumptions_for_the_actual_base() {
    let left = Expression::parse("O(log_log_a(n))").expect("named logarithm");
    let right = Expression::parse("O(log(n))").expect("logarithm");
    let assumptions = GrowthAssumptions::new()
        .with("n", SymbolGrowth::Growing)
        .with("a", SymbolGrowth::FixedLogBase);
    assert_eq!(compare_growth(ThisOrThat::new(&left, &right), &assumptions), Dominance::Undetermined);
    let assumptions = assumptions.with("log_a", SymbolGrowth::FixedLogBase);
    assert_eq!(compare_growth(ThisOrThat::new(&left, &right), &assumptions), Dominance::Equivalent);
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
