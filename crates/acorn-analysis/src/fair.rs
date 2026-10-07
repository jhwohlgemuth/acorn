//! Portable FAIR metadata evidence and assessment.
use super::{Check, CheckCategory, CheckSeverity, Finding};
use crate::constants::fair;
use acorn_core::prelude::{format, Box, String, ToString, Vec};
use acorn_core::{Location, Scheme};
use acorn_schema::pid::{Identifier, PID};
use core::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::EnumIs;

trait ValueExt {
    fn contains(&self) -> bool;
    fn display(&self) -> Option<String>;
    fn find(&self, keys: &[&str]) -> Option<FairEvidenceItem>;
    fn find_value<'a>(&'a self, keys: &[&str], path: String) -> Option<(String, &'a Value)>;
}
/// Identifier from the [RDA FAIR Data Maturity Model](https://doi.org/10.15497/RDA00050).
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, strum::Display, strum::EnumString)]
#[serde(into = "String", try_from = "String")]
pub enum FairIndicatorId {
    /// `RDA-F1-01M`.
    #[strum(serialize = "RDA-F1-01M")]
    RdaF1_01M,
    /// `RDA-F1-01D`.
    #[strum(serialize = "RDA-F1-01D")]
    RdaF1_01D,
    /// `RDA-F1-02M`.
    #[strum(serialize = "RDA-F1-02M")]
    RdaF1_02M,
    /// `RDA-F1-02D`.
    #[strum(serialize = "RDA-F1-02D")]
    RdaF1_02D,
    /// `RDA-F2-01M`.
    #[strum(serialize = "RDA-F2-01M")]
    RdaF2_01M,
    /// `RDA-F3-01M`.
    #[strum(serialize = "RDA-F3-01M")]
    RdaF3_01M,
    /// `RDA-F4-01M`.
    #[strum(serialize = "RDA-F4-01M")]
    RdaF4_01M,
    /// `RDA-A1-01M`.
    #[strum(serialize = "RDA-A1-01M")]
    RdaA1_01M,
    /// `RDA-A1-02M`.
    #[strum(serialize = "RDA-A1-02M")]
    RdaA1_02M,
    /// `RDA-A1-02D`.
    #[strum(serialize = "RDA-A1-02D")]
    RdaA1_02D,
    /// `RDA-A1-03M`.
    #[strum(serialize = "RDA-A1-03M")]
    RdaA1_03M,
    /// `RDA-A1-03D`.
    #[strum(serialize = "RDA-A1-03D")]
    RdaA1_03D,
    /// `RDA-A1-04M`.
    #[strum(serialize = "RDA-A1-04M")]
    RdaA1_04M,
    /// `RDA-A1-04D`.
    #[strum(serialize = "RDA-A1-04D")]
    RdaA1_04D,
    /// `RDA-A1-05D`.
    #[strum(serialize = "RDA-A1-05D")]
    RdaA1_05D,
    /// `RDA-A1.1-01M`.
    #[strum(serialize = "RDA-A1.1-01M")]
    RdaA1_1_01M,
    /// `RDA-A1.1-01D`.
    #[strum(serialize = "RDA-A1.1-01D")]
    RdaA1_1_01D,
    /// `RDA-A1.2-01D`.
    #[strum(serialize = "RDA-A1.2-01D")]
    RdaA1_2_01D,
    /// `RDA-A2-01M`.
    #[strum(serialize = "RDA-A2-01M")]
    RdaA2_01M,
    /// `RDA-I1-01M`.
    #[strum(serialize = "RDA-I1-01M")]
    RdaI1_01M,
    /// `RDA-I1-01D`.
    #[strum(serialize = "RDA-I1-01D")]
    RdaI1_01D,
    /// `RDA-I1-02M`.
    #[strum(serialize = "RDA-I1-02M")]
    RdaI1_02M,
    /// `RDA-I1-02D`.
    #[strum(serialize = "RDA-I1-02D")]
    RdaI1_02D,
    /// `RDA-I2-01M`.
    #[strum(serialize = "RDA-I2-01M")]
    RdaI2_01M,
    /// `RDA-I2-01D`.
    #[strum(serialize = "RDA-I2-01D")]
    RdaI2_01D,
    /// `RDA-I3-01M`.
    #[strum(serialize = "RDA-I3-01M")]
    RdaI3_01M,
    /// `RDA-I3-01D`.
    #[strum(serialize = "RDA-I3-01D")]
    RdaI3_01D,
    /// `RDA-I3-02M`.
    #[strum(serialize = "RDA-I3-02M")]
    RdaI3_02M,
    /// `RDA-I3-02D`.
    #[strum(serialize = "RDA-I3-02D")]
    RdaI3_02D,
    /// `RDA-I3-03M`.
    #[strum(serialize = "RDA-I3-03M")]
    RdaI3_03M,
    /// `RDA-I3-04M`.
    #[strum(serialize = "RDA-I3-04M")]
    RdaI3_04M,
    /// `RDA-R1-01M`.
    #[strum(serialize = "RDA-R1-01M")]
    RdaR1_01M,
    /// `RDA-R1.1-01M`.
    #[strum(serialize = "RDA-R1.1-01M")]
    RdaR1_1_01M,
    /// `RDA-R1.1-02M`.
    #[strum(serialize = "RDA-R1.1-02M")]
    RdaR1_1_02M,
    /// `RDA-R1.1-03M`.
    #[strum(serialize = "RDA-R1.1-03M")]
    RdaR1_1_03M,
    /// `RDA-R1.2-01M`.
    #[strum(serialize = "RDA-R1.2-01M")]
    RdaR1_2_01M,
    /// `RDA-R1.2-02M`.
    #[strum(serialize = "RDA-R1.2-02M")]
    RdaR1_2_02M,
    /// `RDA-R1.3-01M`.
    #[strum(serialize = "RDA-R1.3-01M")]
    RdaR1_3_01M,
    /// `RDA-R1.3-01D`.
    #[strum(serialize = "RDA-R1.3-01D")]
    RdaR1_3_01D,
    /// `RDA-R1.3-02M`.
    #[strum(serialize = "RDA-R1.3-02M")]
    RdaR1_3_02M,
    /// `RDA-R1.3-02D`.
    #[strum(serialize = "RDA-R1.3-02D")]
    RdaR1_3_02D,
}
/// One of the four [FAIR principles](https://www.go-fair.org/fair-principles/).
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FairPrinciple {
    /// [Accessible](https://www.go-fair.org/fair-principles/a1-meta-data-retrievable-identifier-standardised-communication-protocol/).
    Accessible,
    /// [Findable](https://www.go-fair.org/fair-principles/f1-meta-data-assigned-globally-unique-persistent-identifiers/).
    Findable,
    /// [Interoperable](https://www.go-fair.org/fair-principles/i1-meta-data-use-formal-accessible-shared-broadly-applicable-language-knowledge-representation/).
    Interoperable,
    /// [Reusable](https://www.go-fair.org/fair-principles/r1-meta-data-richly-described-plurality-accurate-relevant-attributes/).
    Reusable,
}
/// Priority assigned by the [RDA FAIR Data Maturity Model](https://doi.org/10.15497/RDA00050).
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FairPriority {
    /// Indicator that is essential under most circumstances.
    Essential,
    /// Indicator that substantially increases FAIR support.
    Important,
    /// Indicator that is useful but not indispensable.
    Useful,
}
/// Evidence state for an [RDA FAIR indicator](https://doi.org/10.15497/RDA00050).
#[derive(Clone, Copy, Debug, Deserialize, EnumIs, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FairState {
    /// Available evidence is insufficient to determine support.
    Indeterminate,
    /// The indicator does not apply to this resource.
    NotApplicable,
    /// Available evidence supports the indicator.
    Supported,
    /// Available evidence explicitly contradicts the indicator.
    Unsupported,
}
/// Counts of FAIR findings by evidence state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FairCounts {
    /// Findings that could not be determined.
    pub indeterminate: usize,
    /// Findings that do not apply.
    pub not_applicable: usize,
    /// Findings supported by evidence.
    pub supported: usize,
    /// Findings contradicted by evidence.
    pub unsupported: usize,
}
/// Result for one [RDA FAIR indicator](https://doi.org/10.15497/RDA00050).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FairDetails {
    /// Evidence supporting the result, when available.
    pub evidence: Option<String>,
    /// Evaluated RDA indicator.
    pub indicator: FairIndicator,
    /// Metadata path associated with the result.
    pub locator: Option<String>,
    /// Concrete action that can improve support.
    pub remediation: String,
    /// Identity of the assessed resource.
    pub resource: String,
    /// Evidence state.
    pub state: FairState,
    /// URI whose resolution can strengthen the result.
    pub target_uri: Option<String>,
}
/// Normalized local evidence used to evaluate [RDA FAIR indicators](https://doi.org/10.15497/RDA00050).
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FairEvidence {
    /// Access or landing-page location.
    pub access_uri: Option<FairEvidenceItem>,
    /// Authentication or authorization declaration.
    pub authentication: Option<FairEvidenceItem>,
    /// Declared community metadata standard.
    pub community_standard: Option<FairEvidenceItem>,
    /// Creator or contributor metadata.
    pub creator: Option<FairEvidenceItem>,
    /// Identifier of the described data or digital object.
    pub data_identifier: Option<FairEvidenceItem>,
    /// Declared representation format of the data.
    pub data_format: Option<FairEvidenceItem>,
    /// Descriptive metadata.
    pub description: Option<FairEvidenceItem>,
    /// Direct, machine-retrievable data location.
    pub direct_uri: Option<FairEvidenceItem>,
    /// Standard or machine-readable reuse license.
    pub license: Option<FairEvidenceItem>,
    /// Identifier of the metadata record.
    pub metadata_identifier: Option<FairEvidenceItem>,
    /// Machine-understandable metadata representation.
    pub metadata_format: Option<FairEvidenceItem>,
    /// Provenance information.
    pub provenance: Option<FairEvidenceItem>,
    /// Publisher or repository metadata.
    pub publisher: Option<FairEvidenceItem>,
    /// Qualified relationship to another resource.
    pub qualified_relation: Option<FairEvidenceItem>,
    /// Relationship to another resource.
    pub relation: Option<FairEvidenceItem>,
    /// Stable identity of the assessed resource.
    pub resource: String,
    /// Metadata-retention declaration.
    pub retention: Option<FairEvidenceItem>,
    /// Subject, keyword, or classification metadata.
    pub subject: Option<FairEvidenceItem>,
    /// Human-readable resource title.
    pub title: Option<FairEvidenceItem>,
    /// Controlled vocabulary or vocabulary URI.
    pub vocabulary: Option<FairEvidenceItem>,
}
/// One located item of metadata evidence.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FairEvidenceItem {
    /// Metadata path containing the evidence.
    pub locator: String,
    /// Human-readable evidence value.
    pub value: String,
}
/// Stable definition of an indicator from the [RDA FAIR Data Maturity Model](https://doi.org/10.15497/RDA00050).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FairIndicator {
    /// RDA indicator description.
    pub description: String,
    /// Stable RDA indicator identifier.
    pub id: FairIndicatorId,
    /// FAIR principle identifier, such as `F1` or `R1.3`.
    pub indicator: String,
    /// FAIR principle measured by the indicator.
    pub principle: FairPrinciple,
    /// RDA priority assigned to the indicator.
    pub priority: FairPriority,
    /// Canonical RDA recommendation URL.
    pub url: String,
}
/// FAIR evidence profile for one resource or a pooled batch.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FairProfile {
    /// Accessible evidence counts.
    pub accessible: FairCounts,
    /// Findable evidence counts.
    pub findable: FairCounts,
    /// Interoperable evidence counts.
    pub interoperable: FairCounts,
    /// Profile label or resource identity.
    pub resource: String,
    /// Reusable evidence counts.
    pub reusable: FairCounts,
    /// Counts across all four principles.
    pub total: FairCounts,
}
impl From<FairDetails> for Check {
    fn from(value: FairDetails) -> Self {
        Finding::from(value).into()
    }
}
impl From<FairDetails> for CheckSeverity {
    fn from(value: FairDetails) -> Self {
        match value.state {
            | FairState::Indeterminate => Self::Suggestion,
            | FairState::NotApplicable | FairState::Supported => Self::Info,
            | FairState::Unsupported => Self::Warning,
        }
    }
}
impl FairCounts {
    /// Combine two state-count values.
    pub fn combine(self, other: Self) -> Self {
        Self {
            indeterminate: self.indeterminate.saturating_add(other.indeterminate),
            not_applicable: self.not_applicable.saturating_add(other.not_applicable),
            supported: self.supported.saturating_add(other.supported),
            unsupported: self.unsupported.saturating_add(other.unsupported),
        }
    }
    /// Fraction of applicable indicators for which evidence was determinate.
    pub fn coverage(self) -> Option<f64> {
        let determinate = self.supported.saturating_add(self.unsupported);
        let applicable = determinate.saturating_add(self.indeterminate);
        (applicable > 0).then(|| determinate as f64 / applicable as f64)
    }
    /// Format support and coverage as a compact summary.
    pub fn format(self) -> String {
        let support = self
            .support()
            .map(|value| format!("{:.0}%", value * 100.0))
            .unwrap_or_else(|| "n/a".to_string());
        let coverage = self
            .coverage()
            .map(|value| format!("{:.0}%", value * 100.0))
            .unwrap_or_else(|| "n/a".to_string());
        format!("{support} support; {coverage} coverage")
    }
    /// Fraction of determinate indicators supported by evidence.
    pub fn support(self) -> Option<f64> {
        let determinate = self.supported.saturating_add(self.unsupported);
        (determinate > 0).then(|| self.supported as f64 / determinate as f64)
    }
    fn with_state(self, state: FairState) -> Self {
        match state {
            | FairState::Indeterminate => Self {
                indeterminate: self.indeterminate.saturating_add(1),
                ..self
            },
            | FairState::NotApplicable => Self {
                not_applicable: self.not_applicable.saturating_add(1),
                ..self
            },
            | FairState::Supported => Self {
                supported: self.supported.saturating_add(1),
                ..self
            },
            | FairState::Unsupported => Self {
                unsupported: self.unsupported.saturating_add(1),
                ..self
            },
        }
    }
}
impl FairDetails {
    /// Build one finding by evaluating normalized evidence against an indicator.
    pub fn from_evidence(indicator: FairIndicator, evidence: &FairEvidence) -> Self {
        let attributes = [
            &evidence.title,
            &evidence.description,
            &evidence.creator,
            &evidence.publisher,
            &evidence.subject,
        ];
        let rich = attributes.into_iter().filter(|item| item.is_some()).count();
        let missing = None;
        let selected = match indicator.id {
            | FairIndicatorId::RdaF1_01M
            | FairIndicatorId::RdaF1_02M
            | FairIndicatorId::RdaA1_02M
            | FairIndicatorId::RdaA1_03M
            | FairIndicatorId::RdaA1_04M
            | FairIndicatorId::RdaA1_1_01M => &evidence.metadata_identifier,
            | FairIndicatorId::RdaF1_01D | FairIndicatorId::RdaF1_02D | FairIndicatorId::RdaF3_01M | FairIndicatorId::RdaA1_03D => {
                &evidence.data_identifier
            }
            | FairIndicatorId::RdaF2_01M | FairIndicatorId::RdaR1_01M if rich >= 4 => &evidence.title,
            | FairIndicatorId::RdaF2_01M | FairIndicatorId::RdaR1_01M => &missing,
            | FairIndicatorId::RdaF4_01M
            | FairIndicatorId::RdaA1_01M
            | FairIndicatorId::RdaA1_02D
            | FairIndicatorId::RdaA1_04D
            | FairIndicatorId::RdaA1_1_01D => &evidence.access_uri,
            | FairIndicatorId::RdaA1_05D => &evidence.direct_uri,
            | FairIndicatorId::RdaA1_2_01D => &evidence.authentication,
            | FairIndicatorId::RdaA2_01M => &evidence.retention,
            | FairIndicatorId::RdaI1_01M | FairIndicatorId::RdaI1_02M | FairIndicatorId::RdaR1_3_01M | FairIndicatorId::RdaR1_3_02M => {
                &evidence.metadata_format
            }
            | FairIndicatorId::RdaI1_01D
            | FairIndicatorId::RdaI1_02D
            | FairIndicatorId::RdaI2_01D
            | FairIndicatorId::RdaR1_3_01D
            | FairIndicatorId::RdaR1_3_02D => &evidence.data_format,
            | FairIndicatorId::RdaI2_01M => &evidence.vocabulary,
            | FairIndicatorId::RdaI3_01M | FairIndicatorId::RdaI3_01D | FairIndicatorId::RdaI3_02M => &evidence.relation,
            | FairIndicatorId::RdaI3_02D | FairIndicatorId::RdaI3_03M | FairIndicatorId::RdaI3_04M => &evidence.qualified_relation,
            | FairIndicatorId::RdaR1_1_01M | FairIndicatorId::RdaR1_1_02M | FairIndicatorId::RdaR1_1_03M => &evidence.license,
            | FairIndicatorId::RdaR1_2_01M | FairIndicatorId::RdaR1_2_02M => &evidence.provenance,
        };
        let (observed_state, item) = match selected {
            | Some(value) => (FairState::Supported, Some(value)),
            | None => (FairState::Indeterminate, None),
        };
        let state = match (indicator.id, item) {
            | (FairIndicatorId::RdaF1_01M | FairIndicatorId::RdaF1_01D | FairIndicatorId::RdaF1_02M | FairIndicatorId::RdaF1_02D, Some(item))
                if Identifier::find_all(&item.value)
                    .into_iter()
                    .any(|identifier| identifier.kind != PID::URL) =>
            {
                FairState::Supported
            }
            | (FairIndicatorId::RdaA1_04M | FairIndicatorId::RdaA1_04D | FairIndicatorId::RdaA1_1_01M | FairIndicatorId::RdaA1_1_01D, Some(item))
                if matches!(Location::from(item.value.as_str()).scheme(), Scheme::HTTP | Scheme::HTTPS) =>
            {
                FairState::Supported
            }
            | (FairIndicatorId::RdaA1_2_01D, Some(item))
                if {
                    let value = item.value.to_ascii_lowercase();
                    fair::OPEN_ACCESS_TERMS.iter().any(|term| value.contains(term))
                } =>
            {
                FairState::NotApplicable
            }
            | (FairIndicatorId::RdaR1_1_02M, Some(item))
                if {
                    let value = item.value.to_ascii_lowercase();
                    value.contains("://") || fair::LICENSE_TOKENS.iter().any(|term| value.contains(term))
                } =>
            {
                FairState::Supported
            }
            | (FairIndicatorId::RdaR1_1_03M, Some(item)) if item.value.contains("://") => FairState::Supported,
            | (
                FairIndicatorId::RdaA1_02M
                | FairIndicatorId::RdaA1_02D
                | FairIndicatorId::RdaA1_03M
                | FairIndicatorId::RdaA1_03D
                | FairIndicatorId::RdaA1_05D,
                _,
            )
            | (FairIndicatorId::RdaF1_01M | FairIndicatorId::RdaF1_01D | FairIndicatorId::RdaF1_02M | FairIndicatorId::RdaF1_02D, Some(_))
            | (FairIndicatorId::RdaA1_04M | FairIndicatorId::RdaA1_04D | FairIndicatorId::RdaA1_1_01M | FairIndicatorId::RdaA1_1_01D, Some(_))
            | (FairIndicatorId::RdaR1_1_02M | FairIndicatorId::RdaR1_1_03M, Some(_)) => FairState::Indeterminate,
            | _ => observed_state,
        };
        let remediation = match state {
            | FairState::Supported => format!("Evidence supports {}", indicator.id),
            | FairState::Indeterminate => format!("Provide evidence for {}: {}", indicator.id, indicator.description),
            | FairState::NotApplicable => format!("{} does not apply", indicator.id),
            | FairState::Unsupported => format!("Resolve adverse evidence for {}", indicator.id),
        };
        let target_uri = indicator.id.uri_from(evidence);
        Self {
            evidence: item.map(|item| item.value.clone()),
            indicator,
            locator: item.map(|item| item.locator.clone()),
            remediation,
            resource: evidence.resource.clone(),
            state,
            target_uri,
        }
    }
    /// Format compact metadata shared by terse and tabular output.
    pub fn metadata(&self) -> String {
        format!(
            "rule={} state={} priority={} indicator={} principle={}",
            self.indicator.id, self.state, self.indicator.priority, self.indicator.indicator, self.indicator.principle
        )
    }
}
impl FairEvidence {
    /// Normalize serialized metadata from a supported standard into FAIR evidence.
    pub fn from_value(resource: impl Into<String>, standard: impl Into<String>, value: &Value) -> Self {
        let fallback_resource = resource.into();
        let standard = standard.into();
        let access_uri = value.find(&fair::ACCESS_URI_NAMES);
        let data_identifier = value.find(&fair::DATA_IDENTIFIER_NAMES);
        let direct_uri = value.find(&fair::DIRECT_URI_NAMES);
        let resource = data_identifier
            .as_ref()
            .map(|item| item.value.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(fallback_resource);
        Self {
            access_uri,
            authentication: value.find(&fair::AUTHENTICATION_NAMES),
            community_standard: Some(FairEvidenceItem {
                locator: "$schema".to_string(),
                value: standard,
            }),
            creator: value.find(&fair::CREATOR_NAMES),
            data_format: value.find(&fair::DATA_FORMAT_NAMES),
            data_identifier: data_identifier.clone(),
            description: value.find(&fair::DESCRIPTION_NAMES),
            direct_uri,
            license: value.find(&fair::LICENSE_NAMES),
            metadata_format: Some(FairEvidenceItem {
                locator: "$".to_string(),
                value: "JSON or YAML".to_string(),
            }),
            metadata_identifier: value.find(&fair::METADATA_IDENTIFIER_NAMES),
            provenance: value.find(&fair::PROVENANCE_NAMES),
            publisher: value.find(&fair::PUBLISHER_NAMES),
            qualified_relation: value.find(&fair::QUALIFIED_RELATION_NAMES),
            relation: value.find(&fair::RELATION_NAMES),
            resource,
            retention: value.find(&fair::RETENTION_NAMES),
            subject: value.find(&fair::SUBJECT_NAMES),
            title: value.find(&fair::TITLE_NAMES),
            vocabulary: value.find(&fair::VOCABULARY_NAMES).filter(|item| item.value.contains("://")),
        }
    }
    /// Evaluate all 41 RDA indicators in their canonical order.
    pub fn assess(&self) -> Vec<FairDetails> {
        indicator()
            .into_iter()
            .map(|indicator| FairDetails::from_evidence(indicator, self))
            .collect()
    }
}
impl FairIndicatorId {
    /// Return whether text is valid as a portable analyzer rule identifier.
    pub(crate) fn validate(value: &str) -> bool {
        value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ['.', '-', '_'].contains(&character))
    }
    /// Select the URI whose resolution can provide evidence for this indicator.
    pub fn uri_from(self, evidence: &FairEvidence) -> Option<String> {
        match self {
            | Self::RdaA1_02M | Self::RdaA1_03M => evidence.metadata_identifier.as_ref().map(|item| item.value.clone()),
            | Self::RdaA1_02D | Self::RdaA1_03D | Self::RdaA1_05D => evidence
                .direct_uri
                .as_ref()
                .or(evidence.access_uri.as_ref())
                .map(|item| item.value.clone()),
            | _ => None,
        }
    }
}
impl TryFrom<String> for FairIndicatorId {
    type Error = strum::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl FairPrinciple {
    /// Count findings associated with this principle by evidence state.
    pub fn filter_and_count(self, findings: &[FairDetails]) -> FairCounts {
        findings
            .iter()
            .filter(|finding| finding.indicator.principle == self)
            .fold(FairCounts::default(), |counts, finding| counts.with_state(finding.state))
    }
}
impl fmt::Display for FairPrinciple {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Accessible => "accessible",
            | Self::Findable => "findable",
            | Self::Interoperable => "interoperable",
            | Self::Reusable => "reusable",
        })
    }
}
impl fmt::Display for FairPriority {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Essential => "essential",
            | Self::Important => "important",
            | Self::Useful => "useful",
        })
    }
}
impl FairProfile {
    /// Build a profile from findings for one resource.
    pub fn from_findings(resource: impl Into<String>, findings: &[FairDetails]) -> Self {
        let accessible = FairPrinciple::Accessible.filter_and_count(findings);
        let findable = FairPrinciple::Findable.filter_and_count(findings);
        let interoperable = FairPrinciple::Interoperable.filter_and_count(findings);
        let reusable = FairPrinciple::Reusable.filter_and_count(findings);
        Self {
            accessible,
            findable,
            interoperable,
            resource: resource.into(),
            reusable,
            total: accessible.combine(findable).combine(interoperable).combine(reusable),
        }
    }
    /// Pool profiles without weighting indicators by priority.
    pub fn pooled(resource: impl Into<String>, profiles: &[Self]) -> Self {
        let combined = profiles.iter().fold(Self::empty(resource.into()), |profile, value| Self {
            accessible: profile.accessible.combine(value.accessible),
            findable: profile.findable.combine(value.findable),
            interoperable: profile.interoperable.combine(value.interoperable),
            resource: profile.resource,
            reusable: profile.reusable.combine(value.reusable),
            total: profile.total.combine(value.total),
        });
        combined
    }
    fn empty(resource: String) -> Self {
        Self {
            accessible: FairCounts::default(),
            findable: FairCounts::default(),
            interoperable: FairCounts::default(),
            resource,
            reusable: FairCounts::default(),
            total: FairCounts::default(),
        }
    }
}
impl fmt::Display for FairState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Indeterminate => "indeterminate",
            | Self::NotApplicable => "not-applicable",
            | Self::Supported => "supported",
            | Self::Unsupported => "unsupported",
        })
    }
}
impl From<FairDetails> for Finding {
    fn from(value: FairDetails) -> Self {
        let context = value.evidence.clone();
        let locator = value.locator.clone().or_else(|| Some(value.indicator.id.to_string()));
        let message = value.remediation.clone();
        let rule_id = value.indicator.id.to_string();
        let uri = Some(value.resource.clone());
        let severity = CheckSeverity::from(value.clone());
        let success = value.state.is_supported() || value.state.is_not_applicable();
        Self {
            category: CheckCategory::Fair,
            context,
            fair: Some(Box::new(value)),
            locator,
            message,
            rule_id,
            severity,
            success,
            uri,
        }
    }
}
impl<'a> From<&'a FairIndicator> for &'a str {
    fn from(value: &'a FairIndicator) -> Self {
        value.description.as_str()
    }
}
impl From<FairIndicator> for String {
    fn from(value: FairIndicator) -> Self {
        format!("{} — {:?} — {:?}", value.id, value.principle, value.priority)
    }
}
impl From<FairIndicatorId> for String {
    fn from(value: FairIndicatorId) -> Self {
        value.to_string()
    }
}
impl ValueExt for Value {
    fn contains(&self) -> bool {
        match self {
            | Value::Array(values) => !values.is_empty(),
            | Value::Null => false,
            | Value::Object(values) => !values.is_empty(),
            | Value::String(value) => !value.trim().is_empty(),
            | _ => true,
        }
    }
    fn display(&self) -> Option<String> {
        match self {
            | Value::String(value) if !value.trim().is_empty() => Some(value.clone()),
            | Value::Array(values) if !values.is_empty() => values.iter().find_map(ValueExt::display),
            | Value::Bool(value) => Some(value.to_string()),
            | Value::Number(value) => Some(value.to_string()),
            | Value::Object(values) if !values.is_empty() => values.values().find_map(ValueExt::display),
            | _ => None,
        }
    }
    fn find(&self, keys: &[&str]) -> Option<FairEvidenceItem> {
        self.find_value(keys, "$".to_string())
            .and_then(|(locator, value)| value.display().map(|value| FairEvidenceItem { locator, value }))
    }
    fn find_value<'a>(&'a self, keys: &[&str], path: String) -> Option<(String, &'a Value)> {
        match self {
            | Value::Object(values) => values
                .iter()
                .find(|(key, value)| keys.iter().any(|candidate| key.eq_ignore_ascii_case(candidate)) && value.contains())
                .map(|(key, value)| (format!("{path}.{key}"), value))
                .or_else(|| values.iter().find_map(|(key, value)| value.find_value(keys, format!("{path}.{key}")))),
            | Value::Array(values) => values
                .iter()
                .enumerate()
                .find_map(|(index, value)| value.find_value(keys, format!("{path}[{index}]"))),
            | _ => None,
        }
    }
}
/// Load the ordered RDA FAIR Data Maturity Model indicator registry.
pub(crate) fn indicator() -> Vec<FairIndicator> {
    serde_json::from_str(include_str!("../assets/fair.json")).unwrap_or_default()
}
