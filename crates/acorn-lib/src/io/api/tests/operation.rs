use crate::io::api::operation::{authorize, InvocationContext, OperationEffect, OperationEffects, OperationPolicyError};

#[test]
fn test_authorization_enforces_shared_effect_policy() {
    let effects = [OperationEffect::FilesystemWrite, OperationEffect::NetworkRead]
        .into_iter()
        .collect::<OperationEffects>();
    assert_eq!(
        authorize(&effects, &InvocationContext::default()),
        Err(OperationPolicyError::MutationApprovalRequired)
    );
    assert_eq!(
        authorize(
            &effects,
            &InvocationContext {
                allow_mutation: true,
                offline: true,
                ..InvocationContext::default()
            }
        ),
        Err(OperationPolicyError::Offline)
    );
    assert!(authorize(
        &effects,
        &InvocationContext {
            allow_mutation: true,
            ..InvocationContext::default()
        }
    )
    .is_ok());
}
#[test]
fn test_effect_collection_is_idempotent() {
    let effects = [OperationEffect::Mutation, OperationEffect::Mutation]
        .into_iter()
        .collect::<OperationEffects>();
    assert!(effects.mutation);
    assert_eq!(
        effects,
        OperationEffects {
            mutation: true,
            ..OperationEffects::default()
        }
    );
}
