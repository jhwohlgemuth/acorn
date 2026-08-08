use crate::prelude::*;
#[cfg(feature = "std")]
use crate::schema::agent::Model;
use crate::schema::namespaces::{codemeta, schema_org};
use crate::util::{LinkedData, StringInterpolation, ToMarkdown};
use bon::Builder;
use core::str::FromStr;
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Description of obstacles to shared access
///
/// Data availability is strongly related to the concept of "openness" and the "presumed open principle" for data.
///
/// Where openness is characterized as open, public, shared, or closed, availability takes a more direct approach to
/// describing the actual availability of data, which may be open but still unavailable due to other factors
/// (e.g., technical barriers, legal barriers, etc.).
#[derive(Clone, Debug, Default, Deserialize_repr, Display, Serialize_repr, PartialEq, PartialOrd, JsonSchema)]
#[repr(u8)]
#[serde(deny_unknown_fields)]
pub enum Availability {
    /// Data is not available by any means
    #[default]
    Unavailable = 0,
    /// Data is available but access is restricted (partial availability)
    Restricted = 1,
    /// Data is available and access is unrestricted (open source)
    Unrestricted = 2,
}
/// Describe the data that a system can process, use, and depend on
///
/// The attribute of data that designates "real" vs "synthetic" is intended to capture the ***origin*** of the data, which can have implications for its quality, reliability, and applicability to different tasks.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Data {
    /// Data that has been collected from natural events
    Real(DataDescription),
    /// Data that has been artificially created by computer algorithms
    Synthetic(DataDescription),
    /// Opaque data artifact
    #[cfg(feature = "std")]
    Model(Box<Model>),
}
/// Framework for assessing the readiness of a data set
///
/// Adapted from the concept of "data readiness levels" as described by N. Lawrence in [Data Readiness Levels](http://arxiv.org/abs/1705.02245).
#[derive(Clone, Debug, Deserialize, Display, Serialize, JsonSchema)]
pub enum DataReadinessLevel {
    /// Bottom of Band C. Hearsay data — existence is not yet verified. Signs include statements such
    /// as "The sales department should have a record of that." Problems may include whether the data
    /// is actually being recorded, the format in which it is recorded, privacy or legal constraints,
    /// and topology limitations (e.g., data distributed across many devices).
    #[display("Level E: Hearsay")]
    E,
    /// Top of Band C. Data is machine readable and ethical procedures for data handling have been
    /// addressed. It is ready to be loaded into analysis software or made available via a data
    /// repository. Reaching D often requires significant bespoke software and human understanding
    /// of systems, ethics, and the law.
    #[display("Level D: Accessible")]
    D,
    /// Beginning of Band C. Data is accessible and loaded into analysis software but its faithfulness
    /// and representation have not yet been characterized. Missing values, noise, data entry errors,
    /// unit correctness, and collection bias have not been assessed.
    #[display("Level C: Loaded")]
    C,
    /// End of Band B. The analyst understands how faithful the data is to its original description.
    /// A broad idea of limitations in the data is present and an intuition about what may really be
    /// possible with the data set is beginning to form. Getting to this point is often the most
    /// expensive part of a data project.
    #[display("Level B: Faithful")]
    B,
    /// Data in context. The data set has been assessed as appropriate for a specific task or question.
    /// Any remedial steps have been taken and the data is ready to be deployed in the given context.
    /// A data set may be A1 for one question but only B1 for a different question; the task definition
    /// is an important pre-requisite for this band.
    #[display("Level A: Appropriate")]
    A,
}
/// Levels adapted from NASA Earth Observing System Data and Information System ([EOSDIS]) data processing levels,
/// which are intended to describe the level of processing that has been applied to a given data set.
/// These levels are widely used in the Earth science community and provide a useful framework for understanding the maturity and usability of data products.
///
/// See <https://www.earthdata.nasa.gov/learn/earth-observation-data-basics/data-processing-levels> for more information.
///
/// [EOSDIS]: https://www.earthdata.nasa.gov/about/esdis/eosdis
#[derive(Clone, Debug, Deserialize, Display, Serialize, JsonSchema)]
pub enum GeospatialDataProcessingLevel {
    /// Reconstructed, unprocessed ("raw") instrument and payload data at full resolution,
    /// with any and all communications artifacts (e.g., synchronization frames,
    /// communications headers, duplicate data) removed.
    #[display("Level 0: Raw")]
    L0,
    /// Level 1A (L1A) data are reconstructed, unprocessed instrument data at full
    /// resolution, time-referenced, and annotated with ancillary information.
    /// This ancillary information can include radiometric and geometric calibration
    /// coefficients and georeferencing parameters (e.g., platform ephemeris).
    #[display("Level 1A: Annotated")]
    L1A,
    /// L1B data are L1A data that have been processed to instrument units
    /// (not all instruments have L1B source data).
    #[display("Level 1B: Processed Annotated")]
    L1B,
    /// L1C data are L1B data that include new variables to describe the spectra.
    /// These variables allow the user to identify which L1C channels have been
    /// copied directly from the L1B and which have been synthesized from L1B and why.
    #[display("Level 1C: Spectral Variables")]
    L1C,
    /// Derived geophysical variables at the same resolution and location as
    /// L1 source data.
    #[display("Level 2: Derived Geophysical")]
    L2,
    /// L2A data contains information derived from the geolocated instrument data,
    /// such as ground elevation, highest and lowest surface return elevations,
    /// energy quantile heights ("relative height" metrics), and other
    /// waveform-derived metrics describing the intercepted surface.
    #[display("Level 2A: Derived Surface")]
    L2A,
    /// L2B data are L2A data that have been processed to instrument units
    /// (not all instruments will have a L2B equivalent).
    #[display("Level 2B: Processed Derived Surface")]
    L2B,
    /// Variables mapped on uniform space-time grid scales, usually with some
    /// completeness and consistency.
    #[display("Level 3: Gridded")]
    L3,
    /// L3A data are generally periodic summaries (weekly, 10-day, monthly)
    /// of L2 products.
    #[display("Level 3A: Periodic Summaries")]
    L3A,
    /// Model output or results from analyses of lower-level data
    /// (e.g., variables derived from multiple measurements).
    #[display("Level 4: Model Output")]
    L4,
}
/// Specific type or form of data that a system can process and learn from
#[derive(Clone, Debug, Default, Display, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    /// Textual data that includes text and tabular data
    #[default]
    #[display("text")]
    Text,
    /// Audio data such as MPEG
    #[display("audio")]
    Audio,
    /// Video data such as MP4
    #[display("video")]
    Video,
    /// Signal data
    #[display("signal")]
    Signal,
    /// Graph data or generally data with relational structure
    #[display("graph")]
    Graph,
    /// Image data such as PNG, JPEG
    #[display("image")]
    Image,
    /// PDF document data
    #[display("pdf")]
    Pdf,
}
/// Framework for partitioning data by quality using metaphor of mined metals
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
pub enum Quality {
    /// Raw and unprocessed
    Raw = 0,
    /// Ingested
    Bronze = 1,
    /// Processed in some form short of being "AI ready" (e.g., large scale consumption)
    Silver = 2,
    /// AI-ready
    Gold = 3,
    /// AI-ready data that is adapted to a specific application and/or subjected to additional processing
    Platinum = 4,
}
/// Common attributes of data
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init)]
pub struct DataDescription {
    /// Linked data (e.g., JSON-LD) context for data description
    #[serde(rename = "@context")]
    pub context: Option<DataDescriptionContext>,
    /// Linked data (e.g., JSON-LD) type for data description
    #[serde(rename = "@type")]
    pub description_type: Option<String>,
    /// Replaces ADEPT concept of "openness"
    pub availability: Option<Availability>,
    /// License that governs use of data
    // TODO: validate with is_license
    pub license: Option<String>,
    /// Modality of data
    pub modality: Option<Modality>,
    /// Quality of data
    pub quality: Option<Quality>,
}
/// Linked data (e.g., JSON-LD) context for data description
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init, on(String, into))]
pub struct DataDescriptionContext {
    pub(crate) availability: String,
    pub(crate) license: String,
    pub(crate) modality: String,
    pub(crate) quality: String,
}
impl Default for DataDescription {
    fn default() -> Self {
        DataDescription::init().build()
    }
}
impl Default for DataDescriptionContext {
    fn default() -> Self {
        DataDescriptionContext::init()
            .availability(schema_org("DefinedTerm"))
            .license(codemeta("softwareVersion"))
            .modality(schema_org("DefinedTerm"))
            .quality(schema_org("DefinedTerm"))
            .build()
    }
}
impl ToMarkdown for Data {
    fn to_markdown(&self) -> String {
        match self {
            | Self::Real(description) => format!("- Real{}", description.to_markdown()),
            | Self::Synthetic(description) => format!("- Synthetic{}", description.to_markdown()),
            #[cfg(feature = "std")]
            | Self::Model(model) => match model.as_ref() {
                | Model::SLM(details) => format!("- Model\n  - Kind: SLM\n{}", details.to_markdown().with_indent(2)),
                | Model::LLM(details) => format!("- Model\n  - Kind: LLM\n{}", details.to_markdown().with_indent(2)),
            },
        }
    }
}
impl ToMarkdown for DataDescription {
    fn to_markdown(&self) -> String {
        let lines = [
            self.availability.as_ref().map(|value| format!("Availability: {value}")),
            self.license.as_ref().map(|value| format!("License: {value}")),
            self.modality.as_ref().map(|value| format!("Modality: {value}")),
            self.quality.as_ref().map(|value| format!("Quality: {}", value)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        if lines.is_empty() {
            String::new()
        } else {
            format!("\n{}", lines.to_markdown().trim_start().with_indent(2))
        }
    }
}
impl LinkedData for DataDescription {
    fn with_context(&self) -> Self {
        Self {
            context: Some(DataDescriptionContext::default()),
            description_type: None,
            ..self.clone()
        }
    }
}
impl FromStr for GeospatialDataProcessingLevel {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            | "LV0" | "L0" | "0" => Ok(Self::L0),
            | "LV1A" | "L1A" | "1A" => Ok(Self::L1A),
            | "LV1B" | "L1B" | "1B" => Ok(Self::L1B),
            | "LV1C" | "L1C" | "1C" => Ok(Self::L1C),
            | "LV2" | "L2" | "2" => Ok(Self::L2),
            | "LV2A" | "L2A" | "2A" => Ok(Self::L2A),
            | "LV2B" | "L2B" | "2B" => Ok(Self::L2B),
            | "LV3" | "L3" | "3" => Ok(Self::L3),
            | "LV3A" | "L3A" | "3A" => Ok(Self::L3A),
            | "LV4" | "L4" | "4" => Ok(Self::L4),
            | _ => Err(format!("unknown GeospatialDataProcessingLevel: {value}")),
        }
    }
}
