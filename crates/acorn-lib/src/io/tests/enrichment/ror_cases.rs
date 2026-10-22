use super::*;

#[test]
fn test_ror_selection_preserves_exact_choices_and_passive_results() {
    let activity = ResearchActivity {
        contact: ContactPoint {
            organization: "Example Laboratory".to_string(),
            ..ContactPoint::default()
        },
        ..ResearchActivity::default()
    };
    let records = vec![
        ror_record("https://ror.org/01first", "first.org"),
        ror_record("https://ror.org/02second", "second.org"),
    ];
    let (unchanged, conflict, choices) = activity.select_ror("Example Laboratory", records.clone().into());
    assert!(unchanged.meta.ror.is_none());
    assert!(conflict.is_none());
    let decision = ResearchActivity::ror_decision(&Location::from("activity.json"), &choices)
        .expect("decision")
        .expect("exact choices");
    assert_eq!(decision.candidates.len(), 2);
    assert_eq!(decision.origin, EnrichmentOrigin::Provider(Provider::Ror));
    let narrowed = ResearchActivity {
        contact: ContactPoint {
            email: "person@first.org".to_string(),
            ..activity.contact.clone()
        },
        ..activity.clone()
    };
    let (resolved, conflict, choices) = narrowed.select_ror("Example Laboratory", records.into());
    assert_eq!(resolved.meta.ror.unwrap().into_vec(), vec!["https://ror.org/01first".to_string()]);
    assert!(conflict.is_none());
    assert!(choices.is_empty());
    let (missing, conflict, choices) = activity.select_ror("Example Laboratory", Vec::new().into());
    assert!(missing.meta.ror.is_none());
    assert!(conflict.is_none());
    assert!(choices.is_empty());
}
