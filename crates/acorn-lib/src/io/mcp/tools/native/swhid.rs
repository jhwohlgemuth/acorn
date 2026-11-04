//! Bounded local SWHID calculation and verification.
use super::super::{
    BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputLimits, InputReference, RoutedInputPolicy, ToolRegistry,
};
use super::NativeEffect;
use crate::error::ApiResult;
use crate::io::swhid::{self, CalculationKind};
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::SemanticVersion;
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::pid::SWHID;
use color_eyre::eyre::eyre;
use core::str::FromStr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// One local object and optional expected identifier.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SwhidInput {
    /// File, directory, or repository beneath the permitted root.
    pub path: PathBuf,
    /// Object kind used by the direct SWHID command.
    #[serde(default)]
    pub kind: CalculationKind,
    /// Git revision expression or annotated tag.
    pub reference: Option<String>,
    /// Expected identifier for verification.
    pub verify: Option<String>,
}
/// Calculated identifier and whether an expected identifier was checked.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SwhidOutput {
    /// Complete Software Hash Identifier.
    pub identifier: String,
    /// True when an expected identifier matched.
    pub verified: bool,
}
impl SwhidInput {
    /// Calculate or verify using the direct command's shared library operation.
    pub fn execute(self, root: &Path) -> ApiResult<SwhidOutput> {
        RoutedInputPolicy::new(root, InputLimits::default()).and_then(|policy| {
            let source = policy.resolve_path(&self.path);
            source.and_then(|source| {
                let local = match (self.kind, source.is_file(), source.is_dir()) {
                    | (CalculationKind::Auto | CalculationKind::Content, true, _) => {
                        policy.resolve(&InputReference::File(self.path.clone())).map(|_| ())
                    }
                    | (CalculationKind::Auto | CalculationKind::Directory, _, true) => {
                        policy.resolve(&InputReference::Directory(self.path.clone())).and_then(|resolved| {
                            if resolved.failures.is_empty() {
                                Ok(())
                            } else {
                                Err(eyre!("SWHID directory exceeds the bounded input policy"))
                            }
                        })
                    }
                    | (
                        CalculationKind::GitBlob
                        | CalculationKind::GitTree
                        | CalculationKind::Revision
                        | CalculationKind::Release
                        | CalculationKind::Snapshot,
                        _,
                        true,
                    ) => Ok(()),
                    | _ => Err(eyre!("SWHID kind does not match the local object")),
                };
                local.and_then(|()| {
                    swhid::calculate(&source, self.kind, self.reference.as_deref()).and_then(|actual| {
                        self.verify
                            .as_deref()
                            .map(SWHID::from_str)
                            .transpose()
                            .map_err(|why| eyre!("invalid expected SWHID: {why}"))
                            .and_then(|expected| {
                                let mismatch = expected
                                    .as_ref()
                                    .is_some_and(|expected| actual.core_identifier() != expected.core_identifier());
                                if mismatch {
                                    Err(eyre!("SWHID mismatch"))
                                } else {
                                    Ok(SwhidOutput {
                                        identifier: actual.to_string(),
                                        verified: expected.is_some(),
                                    })
                                }
                            })
                    })
                })
            })
        })
    }
}
impl ToolRegistry {
    /// Register the bounded local SWHID operation.
    pub fn register_swhid_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::LocalRead);
        let register = self.register_capability(CapabilityDefinition::new::<SwhidInput, SwhidOutput>(
            format!("{APPLICATION}.swhid"),
            version,
            "Calculate or verify a confirmed local Software Hash Identifier.",
            effects,
        ));
        register.and_then(|registry| {
            let definition = BindingDefinition::new::<SwhidInput, SwhidOutput>(
                format!("native:{APPLICATION}.swhid"),
                format!("{APPLICATION}.swhid"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            registry.register_capability_binding::<SwhidInput, SwhidOutput, _, _>(definition.with_offline_support(), move |input| {
                let root = root.clone();
                async move { input.execute(&root) }
            })
        })
    }
}
