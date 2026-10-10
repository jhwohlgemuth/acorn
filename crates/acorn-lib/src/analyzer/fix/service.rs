use super::{
    async_trait, eyre, format, is_identifier, run_inference, schema_for, stream, validate_options, AnalysisReport, ApiResult, BTreeMap, BTreeSet,
    CheckReference, CheckReferences, Deserialize, FindingReference, Fingerprint, Fix, FixAction, FixAutomation, FixDiagnostic, FixEffects, FixError,
    FixOptions, FixProvenance, FixReport, FixRisk, FixStatus, HydratedPrompt, InferenceBackend, InferencePermissionPolicy, InferenceProvenance,
    InferenceRequest, InferenceResult, JsonSchema, Parse, PromptContext, PromptFileAsset, PromptHydrator, PromptTemplateConfiguration, SafePath,
    Serialize, StreamExt, String, StringConversion, ToString, Validate, Value, Vec, FIX_ENVELOPE_VERSION, MAX_MESSAGE_BYTES,
};
use acorn_core::Noul;
pub(super) trait PendingReferences {
    fn normalize(self, options: &FixOptions) -> ApiResult<Vec<CheckReference>>;
}
pub(super) struct PendingReference<'a> {
    pub(super) check: &'a crate::analyzer::Check,
    pub(super) standard: String,
}
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SuggestedFix {
    pub(super) actions: Vec<FixAction>,
    pub(super) automation: FixAutomation,
    pub(super) confidence: Noul,
    pub(super) effects: FixEffects,
    pub(super) finding_id: String,
    pub(super) id: String,
    pub(super) rationale: String,
    pub(super) risk: FixRisk,
    pub(super) source_fingerprint: String,
    pub(super) status: FixStatus,
    pub(super) summary: String,
    pub(super) target: String,
}
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SuggestionEnvelope {
    pub(super) fixes: Vec<SuggestedFix>,
    pub(super) version: String,
}
#[derive(Serialize)]
struct SuggestionPromptData<'a> {
    findings: &'a [CheckReference],
    version: &'static str,
}
impl AnalysisReport {
    /// Normalize unsuccessful checks into stable references.
    pub fn normalize_checks(&self, options: &FixOptions) -> ApiResult<Vec<CheckReference>> {
        validate_options(options).and_then(|()| {
            self.batches
                .iter()
                .flat_map(|batch| {
                    batch.categories.iter().flat_map(move |category| {
                        category
                            .checks
                            .iter()
                            .filter(|&check| !check.success || options.include_successful)
                            .map(|check| PendingReference {
                                check,
                                standard: batch.standard.to_string(),
                            })
                    })
                })
                .collect::<Vec<_>>()
                .normalize(options)
        })
    }
    /// Suggest fixes through one transport-neutral backend.
    pub async fn suggest_fixes<B>(&self, backend: &B, options: &FixOptions) -> ApiResult<FixReport>
    where
        B: InferenceBackend + Sync,
    {
        match self.normalize_checks(options) {
            | Ok(references) => references.suggest(backend, options).await,
            | Err(why) => Err(why),
        }
    }
}
impl TryFrom<CheckReference> for Fix {
    type Error = CheckReference;
    fn try_from(reference: CheckReference) -> Result<Self, Self::Error> {
        let pair = reference
            .vocabulary_advisory
            .as_deref()
            .zip(reference.uri.clone().or(reference.locator.clone()));
        match pair {
            | Some((advisory, target)) => advisory
                .replacement
                .as_ref()
                .filter(|replacement| advisory.complete && advisory.outcome.is_accepted() && replacement.as_str() != advisory.original)
                .map(|replacement| {
                    let action = FixAction::DocumentChange {
                        description: format!("Use canonical {} vocabulary value", advisory.vocabulary),
                        expected: advisory.original.clone(),
                        replacement: replacement.clone(),
                    };
                    let effects = FixEffects {
                        filesystem_write: true,
                        reversible: true,
                        ..FixEffects::default()
                    };
                    let provenance = FixProvenance {
                        agent: None,
                        backend: "host".to_string(),
                        model: None,
                        prompt_version: FIX_ENVELOPE_VERSION.to_string(),
                        request_id: None,
                    };
                    Fix::init()
                        .actions(vec![action])
                        .automation(FixAutomation::SafeAutomatic)
                        .confidence(Noul::ONE)
                        .effects(effects)
                        .finding(FindingReference::Check(reference.clone()))
                        .id(format!("vocabulary-{}", reference.id))
                        .provenance(provenance)
                        .rationale("The deterministic vocabulary resolver selected one complete, unambiguous canonical value")
                        .risk(FixRisk::Low)
                        .status(FixStatus::Proposed)
                        .summary(format!("Replace '{}' with '{}'", advisory.original, replacement))
                        .target(target)
                        .build()
                })
                .ok_or_else(|| reference.clone()),
            | None => Err(reference),
        }
    }
}
impl Parse<Value> for SuggestionEnvelope {
    fn parse(value: Value) -> ApiResult<Self> {
        serde_json::to_value(schema_for!(Self))
            .map_err(|why| eyre!("Serialize fix response schema — {why}"))
            .and_then(|schema| jsonschema::validator_for(&schema).map_err(|why| eyre!("Compile fix response schema — {why}")))
            .and_then(|validator| {
                validator
                    .validate(&value)
                    .map_err(|why| eyre!("Fix response failed schema validation — {why}"))
            })
            .and_then(|()| serde_json::from_value(value).map_err(|why| eyre!("Deserialize fix response — {why}")))
    }
}
impl TryFrom<InferenceResult> for SuggestionEnvelope {
    type Error = color_eyre::eyre::Report;

    fn try_from(result: InferenceResult) -> Result<Self, Self::Error> {
        match result.finish_reason.is_refusal() {
            | true => Err(eyre!("Inference backend refused the fix suggestion request")),
            | false => result
                .structured
                .map_or_else(
                    || serde_json::from_str(&result.text).map_err(|why| eyre!("Fix response is not valid JSON — {why}")),
                    Ok,
                )
                .and_then(Self::parse),
        }
    }
}
#[async_trait]
impl CheckReferences for Vec<CheckReference> {
    fn batch(self, options: &FixOptions) -> Vec<Vec<CheckReference>> {
        self.into_iter().fold(Vec::new(), |mut batches, reference| {
            let starts_batch = batches.last().is_none_or(|batch: &Vec<CheckReference>| {
                let context_bytes = batch.iter().map(CheckReference::context_bytes).sum::<usize>();
                let exceeds_context_limit = context_bytes.saturating_add(reference.context_bytes()) > options.max_context_bytes;
                let reaches_count_limit = batch.len() >= options.max_findings_per_batch;
                exceeds_context_limit || reaches_count_limit
            });
            match starts_batch {
                | true => batches.push(vec![reference]),
                | false => match batches.last_mut() {
                    | Some(batch) => batch.push(reference),
                    | None => batches.push(vec![reference]),
                },
            }
            batches
        })
    }
    fn build(&self, options: &FixOptions) -> ApiResult<InferenceRequest> {
        let max_items = u32::try_from(self.len()).unwrap_or(u32::MAX);
        let context = PromptContext {
            config: PromptTemplateConfiguration::init()
                .max_items(max_items)
                .min_confidence(options.min_confidence)
                .build(),
            data: SuggestionPromptData {
                findings: self,
                version: FIX_ENVELOPE_VERSION,
            },
        };
        PromptHydrator::hydrate(PromptFileAsset::SuggestFixes, &context)
            .and_then(|prompt| {
                serde_json::to_value(schema_for!(SuggestionEnvelope))
                    .map_err(|why| eyre!("Serialize fix response schema — {why}"))
                    .map(|schema| (prompt, schema))
            })
            .and_then(|(prompt, response_schema)| {
                serde_json::to_string_pretty(&response_schema)
                    .map_err(|why| eyre!("Serialize fix response schema for prompt — {why}"))
                    .map(|schema| (prompt, response_schema, schema))
            })
            .map(|(prompt, response_schema, schema)| InferenceRequest {
                agent: options.agent.clone(),
                model: options.model.clone(),
                permission_policy: InferencePermissionPolicy::DenyMutation,
                prompt: HydratedPrompt {
                    body: format!(
                        "{}\n\n# Response Schema\n\n```json\n{}\n```\n\nReturn valid JSON only that conforms to this schema.",
                        prompt.body, schema
                    ),
                    ..prompt
                },
                response_schema: Some(response_schema),
                timeout: options.timeout,
            })
    }
    async fn suggest<B>(self, backend: &B, options: &FixOptions) -> ApiResult<FixReport>
    where
        B: InferenceBackend + Sync,
    {
        match validate_options(options).and_then(|()| self.validate_with(options)) {
            | Err(why) => Err(why),
            | Ok(()) => {
                let (mut deterministic, inference) =
                    self.into_iter()
                        .fold((Vec::new(), Vec::new()), |(mut deterministic, mut inference), reference| {
                            match Fix::try_from(reference) {
                                | Ok(fix) => deterministic.push(fix),
                                | Err(reference) => inference.push(reference),
                            }
                            (deterministic, inference)
                        });
                let outcomes = stream::iter(inference.batch(options))
                    .then(|batch| suggest_batch(backend, batch, options))
                    .collect::<Vec<_>>()
                    .await;
                let (mut inferred, diagnostics) = outcomes
                    .into_iter()
                    .fold((Vec::new(), Vec::new()), |(mut fixes, mut diagnostics), outcome| {
                        match outcome {
                            | Ok(mut batch) => fixes.append(&mut batch),
                            | Err(diagnostic) => diagnostics.push(diagnostic),
                        }
                        (fixes, diagnostics)
                    });
                deterministic.append(&mut inferred);
                Ok(FixReport::from_parts(deterministic, diagnostics))
            }
        }
    }
    fn validate_with(&self, options: &FixOptions) -> ApiResult<()> {
        self.validate()
            .map_err(|why| eyre!(why))
            .and_then(|()| {
                let unique_ids = self.iter().map(|reference| reference.id.as_str()).collect::<BTreeSet<_>>();
                match unique_ids.len() == self.len() {
                    | true => Ok(()),
                    | false => Err(FixError::RepeatedFinding.into()),
                }
            })
            .and_then(|()| {
                self.iter().try_for_each(|reference| {
                    let excerpt_allowed = reference.context.as_ref().is_none_or(|value| value.len() <= options.max_excerpt_bytes);
                    match (excerpt_allowed, reference.context_bytes() <= options.max_context_bytes) {
                        | (false, _) => Err(FixError::ExcerptExceeded.into()),
                        | (_, false) => Err(FixError::FindingContextExceeded.into()),
                        | _ => Ok(()),
                    }
                })
            })
    }
}
impl PendingReferences for Vec<PendingReference<'_>> {
    fn normalize(mut self, options: &FixOptions) -> ApiResult<Vec<CheckReference>> {
        self.sort_by(|left, right| {
            let left_key = (
                left.standard.as_str(),
                left.check.category.to_string(),
                left.check.uri.as_deref(),
                left.check.locator.as_deref(),
                left.check.index,
                left.check.message.as_str(),
            );
            let right_key = (
                right.standard.as_str(),
                right.check.category.to_string(),
                right.check.uri.as_deref(),
                right.check.locator.as_deref(),
                right.check.index,
                right.check.message.as_str(),
            );
            left_key.cmp(&right_key)
        });
        self.into_iter()
            .enumerate()
            .map(|(index, pending)| {
                let context = pending
                    .check
                    .context
                    .as_deref()
                    .map(|value| value.truncate_utf8(options.max_excerpt_bytes));
                let message = pending.check.message.truncate_utf8(MAX_MESSAGE_BYTES);
                let id = format!("check-{:04}", index.saturating_add(1));
                let advisory = pending.check.vocabulary_advisory.clone();
                let advisory_identity = advisory.as_ref().and_then(|value| serde_json::to_string(value).ok()).unwrap_or_default();
                let identity = [
                    pending.standard.as_str(),
                    &pending.check.category.to_string(),
                    pending.check.uri.as_deref().unwrap_or_default(),
                    pending.check.locator.as_deref().unwrap_or_default(),
                    &pending.check.index.to_string(),
                    pending.check.rule_id.as_deref().unwrap_or_default(),
                    message.as_str(),
                    advisory_identity.as_str(),
                ]
                .join("\u{1f}");
                let excerpt = context.as_deref().unwrap_or(message.as_str());
                let fingerprint = Fingerprint::from_entries([(identity, excerpt.as_bytes())]).to_string();
                let reference = CheckReference {
                    category: pending.check.category.to_string(),
                    context,
                    fingerprint,
                    id,
                    locator: pending.check.locator.clone(),
                    message,
                    rule_id: pending.check.rule_id.clone(),
                    severity: pending.check.severity.to_string(),
                    standard: pending.standard,
                    uri: pending.check.uri.clone(),
                    vocabulary_advisory: advisory,
                };
                reference.validate().map_err(|why| eyre!(why)).map(|()| reference)
            })
            .collect::<ApiResult<Vec<_>>>()
    }
}
async fn suggest_batch<B>(backend: &B, references: Vec<CheckReference>, options: &FixOptions) -> Result<Vec<Fix>, FixDiagnostic>
where
    B: InferenceBackend + Sync,
{
    let finding_ids = references.iter().map(|reference| reference.id.clone()).collect::<Vec<_>>();
    let outcome = match references.build(options) {
        | Ok(request) => match run_inference(backend, &request).await {
            | Ok(result) => {
                let provenance = result.provenance.clone();
                SuggestionEnvelope::try_from(result).and_then(|envelope| validate_envelope(envelope, &references, options, provenance))
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    };
    outcome.map_err(|why| FixDiagnostic {
        finding_ids,
        message: why.to_string().truncate_utf8(MAX_MESSAGE_BYTES),
    })
}
pub(super) fn validate_envelope(
    envelope: SuggestionEnvelope,
    references: &[CheckReference],
    options: &FixOptions,
    provenance: InferenceProvenance,
) -> ApiResult<Vec<Fix>> {
    let reference_map = references
        .iter()
        .map(|reference| (reference.id.as_str(), reference))
        .collect::<BTreeMap<_, _>>();
    let unique_ids = envelope.fixes.iter().map(|fix| fix.id.as_str()).collect::<BTreeSet<_>>();
    let duplicate_ids = unique_ids.len() != envelope.fixes.len();
    let unique_targets = envelope
        .fixes
        .iter()
        .map(|fix| (fix.finding_id.as_str(), fix.target.as_str()))
        .collect::<BTreeSet<_>>();
    let duplicate_targets = unique_targets.len() != envelope.fixes.len();
    match (envelope.version.as_str(), duplicate_ids, duplicate_targets) {
        | (FIX_ENVELOPE_VERSION, false, false) => envelope
            .fixes
            .into_iter()
            .map(|suggested| {
                reference_map
                    .get(suggested.finding_id.as_str())
                    .copied()
                    .ok_or_else(|| eyre!("Fix references unknown finding '{}'", suggested.finding_id))
                    .and_then(|reference| validate_suggested_fix(suggested, reference, options, provenance.clone()))
            })
            .collect(),
        | (_, true, _) => Err(eyre!("Fix response contains duplicate fix identifiers")),
        | (_, _, true) => Err(eyre!("Fix response contains duplicate targets for one finding")),
        | (_, false, false) => Err(eyre!("Unsupported fix envelope version '{}'", envelope.version)),
    }
}
pub(super) fn validate_suggested_fix(
    suggested: SuggestedFix,
    reference: &CheckReference,
    options: &FixOptions,
    provenance: InferenceProvenance,
) -> ApiResult<Fix> {
    let allowed_target = [reference.uri.as_deref(), reference.locator.as_deref(), Some(reference.id.as_str())]
        .into_iter()
        .flatten()
        .any(|target| target == suggested.target);
    let unique_actions = suggested.actions.iter().collect::<BTreeSet<_>>().len() == suggested.actions.len();
    let valid_status = suggested.status.is_proposed() || suggested.status.is_rejected() || suggested.status.is_unsupported();
    let valid_action_count = match suggested.status {
        | FixStatus::Proposed => !suggested.actions.is_empty(),
        | FixStatus::Rejected | FixStatus::Unsupported => suggested.actions.is_empty(),
        | _ => false,
    };
    let valid_confidence = !suggested.status.is_proposed() || suggested.confidence >= options.min_confidence;
    let valid_actions = unique_actions && suggested.actions.validate().is_ok();
    let valid_vocabulary_actions = reference.vocabulary_advisory.as_ref().is_none_or(|advisory| {
        advisory.replacement.is_some() || !suggested.actions.iter().any(|action| matches!(action, FixAction::DocumentChange { .. }))
    });
    let valid_fingerprint = suggested.source_fingerprint == reference.fingerprint;
    let valid_identifier = is_identifier(&suggested.id);
    let valid_target = allowed_target && SafePath::new(&suggested.target).is_ok();
    let valid_description = !suggested.summary.trim().is_empty() && !suggested.rationale.trim().is_empty();
    let validation = (
        valid_identifier,
        valid_fingerprint,
        valid_target,
        valid_confidence,
        valid_status,
        valid_action_count,
        valid_actions,
        valid_vocabulary_actions,
        valid_description,
    );
    match validation {
        | (false, _, _, _, _, _, _, _, _) => Err(FixError::InvalidIdentifier.into()),
        | (_, false, _, _, _, _, _, _, _) => Err(FixError::StaleFingerprint.into()),
        | (_, _, false, _, _, _, _, _, _) => Err(FixError::UnsafeTarget.into()),
        | (_, _, _, false, _, _, _, _, _) => Err(FixError::InvalidConfidence.into()),
        | (_, _, _, _, false, _, _, _, _) => Err(FixError::InvalidStatus.into()),
        | (_, _, _, _, _, false, _, _, _) => Err(FixError::ActionCountMismatch.into()),
        | (_, _, _, _, _, _, false, _, _) | (_, _, _, _, _, _, _, false, _) => Err(FixError::InvalidActions.into()),
        | (_, _, _, _, _, _, _, _, false) => Err(FixError::MissingDescription.into()),
        | _ => Ok(Fix {
            actions: suggested.actions,
            automation: suggested.automation,
            confidence: suggested.confidence,
            effects: suggested.effects,
            finding: FindingReference::Check(reference.clone()),
            id: suggested.id,
            provenance: provenance.into(),
            rationale: suggested.rationale,
            risk: suggested.risk,
            status: suggested.status,
            summary: suggested.summary,
            target: suggested.target,
        }),
    }
}
