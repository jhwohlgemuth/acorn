//! Quantum computing paradigms and hardware definitions
use super::vendor::Vendor;
use crate::OneOrMany;
use acorn_core::prelude::alloc::String;
use bon::Builder;
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Chip or system model name for a quantum processing unit (QPU)
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum Model {
    /// D-Wave Advantage (5000+ qubits)
    #[display("Advantage")]
    #[serde(alias = "advantage")]
    Advantage,
    /// D-Wave Advantage2
    #[display("Advantage2")]
    #[serde(alias = "advantage2", alias = "Advantage 2", alias = "advantage 2")]
    Advantage2,
    /// Rigetti Ankaa series
    #[display("Ankaa")]
    #[serde(alias = "ankaa")]
    Ankaa,
    /// Rigetti Aspen series
    #[display("Aspen")]
    #[serde(alias = "aspen")]
    Aspen,
    /// Google Bristlecone (72 qubits)
    #[display("Bristlecone")]
    #[serde(alias = "bristlecone")]
    Bristlecone,
    /// IBM Condor (1121 qubits)
    #[display("Condor")]
    #[serde(alias = "condor")]
    Condor,
    /// IBM Eagle (127 qubits)
    #[display("Eagle")]
    #[serde(alias = "eagle")]
    Eagle,
    /// IBM Falcon
    #[display("Falcon")]
    #[serde(alias = "falcon")]
    Falcon,
    /// IBM Flamingo
    #[display("Flamingo")]
    #[serde(alias = "flamingo")]
    Flamingo,
    /// Quantinuum H1 trapped ion processor
    #[display("H1")]
    #[serde(alias = "h1", alias = "H 1", alias = "h 1")]
    H1,
    /// Quantinuum H2 trapped ion processor
    #[display("H2")]
    #[serde(alias = "h2", alias = "H 2", alias = "h 2")]
    H2,
    /// IBM Heron (133 qubits)
    #[display("Heron")]
    #[serde(alias = "heron")]
    Heron,
    /// IBM Hummingbird
    #[display("Hummingbird")]
    #[serde(alias = "hummingbird")]
    Hummingbird,
    /// IonQ Aria
    #[display("IonQ Aria")]
    #[serde(alias = "IonQ Aria", alias = "ionq aria", alias = "aria", alias = "Aria")]
    IonQAria,
    /// IonQ Forte
    #[display("IonQ Forte")]
    #[serde(alias = "IonQ Forte", alias = "ionq forte", alias = "forte", alias = "Forte")]
    IonQForte,
    /// IonQ Harmony
    #[display("IonQ Harmony")]
    #[serde(alias = "IonQ Harmony", alias = "ionq harmony", alias = "harmony", alias = "Harmony")]
    IonQHarmony,
    /// IonQ Tempo
    #[display("IonQ Tempo")]
    #[serde(alias = "IonQ Tempo", alias = "ionq tempo", alias = "tempo", alias = "Tempo")]
    IonQTempo,
    /// IBM Osprey (433 qubits)
    #[display("Osprey")]
    #[serde(alias = "osprey")]
    Osprey,
    /// Google Sycamore (53/54 qubits)
    ///
    /// See [Quantum supremacy using a programmable superconducting processor](https://www.nature.com/articles/s41586-019-1666-5) for more information
    #[display("Sycamore")]
    #[serde(alias = "sycamore")]
    Sycamore,
    /// D-Wave 2000Q (2000 qubits)
    #[display("2000Q")]
    #[serde(alias = "2000q", alias = "2000Q")]
    TwoThousandQ,
    /// Google Willow (105 qubits)
    #[display("Willow")]
    #[serde(alias = "willow")]
    Willow,
    /// Other or unspecified quantum processor model
    #[display("{}", _0)]
    Other(String),
}
/// Physical mechanism used to implement a quantum processor.
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Technology {
    /// Photons carry the quantum state.
    Photonic,
    /// Neutral atoms carry the quantum state.
    NeutralAtom,
    /// Superconducting circuits carry the quantum state.
    Superconducting,
    /// Trapped ions carry the quantum state.
    TrappedIon,
    /// Another physical mechanism.
    Other(String),
}
/// Whether a qubit count describes physical or logical qubits.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum QubitType {
    /// A physical qubit implemented by the processor technology.
    Physical,
    /// A logical qubit exposed after error correction or encoding.
    Logical,
}
/// Error correction implemented by a quantum processor configuration.
#[derive(Builder, Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ErrorCorrection {
    /// Name of the code or technique in use.
    pub code: String,
    /// Ratio of physical qubits allocated to each logical qubit, when known.
    pub physical_to_logical_ratio: Option<u32>,
}
/// A typed qubit capacity reported by a quantum processor.
#[derive(Builder, Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Qubit {
    /// Whether the capacity counts physical or logical qubits.
    #[serde(rename = "type")]
    pub qubit_type: QubitType,
    /// Number of qubits of this type.
    pub count: u32,
}
/// Configured quantum processing resources for a hardware requirement or platform.
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields)]
pub struct QuantumResource {
    /// Number of QPUs required, when applicable.
    pub count: Option<u32>,
    /// Chip or system model name.
    pub model: Option<Model>,
    /// Physical implementation technology.
    pub technology: Option<Technology>,
    /// Physical and logical qubit capacities, when known.
    #[builder(into)]
    pub qubits: Option<OneOrMany<Qubit>>,
    /// Configured error correction, if any.
    pub error_correction: Option<ErrorCorrection>,
    /// Whether this resource is required.
    pub required: Option<bool>,
    /// Qubit connectivity topology.
    pub topology: Option<Topology>,
    /// Hardware vendor.
    pub vendor: Option<Vendor>,
}
/// Quantum sensing regime classification used by hardware sensor modalities
///
/// Classical systems include mature technologies where measurement's quantum nature is inconsequential, such as radar and RF communications.
///
/// Hybrid systems sit at the classical/quantum boundary, where superposition and measurement are both essential, such as atom or light interferometry.
///
/// Deep systems represent the deep quantum regime where entanglement is essential to the expected advantage, as in quantum simulation/computing.
///
/// ### References
/// \[1\] J. Kitching et al., "Independent Panel Report for Technical Assessment of NASA and External Quantum Sensing Capabilities," 2023.
#[derive(Clone, Debug, Default, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
#[repr(u8)]
pub enum Regime {
    /// Classical domain technologies (ex., radar and RF communications) where the quantum nature of measurement is inconsequential.
    #[default]
    #[display("classical")]
    #[serde(alias = "classical")]
    Classical = 0,
    /// Classical/quantum boundary technologies (ex., atom interferometry and light interferometry with photon detection) where superposition and measurement both play an essential role.
    #[display("hybrid")]
    #[serde(alias = "hybrid")]
    Hybrid = 1,
    /// Deep quantum regime technologies where entanglement is essential for the promised quantum advantage.
    #[display("deep")]
    #[serde(alias = "deep")]
    Deep = 2,
}
/// Qubit connectivity topology
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum Topology {
    /// IBM heavy-hexagonal coupling map
    #[display("heavy-hex")]
    #[serde(alias = "heavy-hex", alias = "heavy_hex", alias = "heavyHex", alias = "heavyhex")]
    HeavyHex,
    /// All-to-all qubit connectivity (e.g., trapped ion systems)
    #[display("all-to-all")]
    #[serde(alias = "all-to-all", alias = "all_to_all", alias = "allToAll")]
    AllToAll,
    /// 2D grid or square lattice (e.g., Google Sycamore)
    #[display("grid")]
    #[serde(alias = "grid", alias = "square lattice", alias = "square_lattice", alias = "SquareLattice")]
    Grid,
    /// Ring topology
    #[display("ring")]
    #[serde(alias = "ring")]
    Ring,
    /// Linear chain topology
    #[display("linear chain")]
    #[serde(alias = "linear chain", alias = "linear_chain", alias = "LinearChain")]
    LinearChain,
    /// Star topology with a central coupler qubit
    #[display("star")]
    #[serde(alias = "star")]
    Star,
    /// D-Wave Pegasus graph topology
    ///
    /// See [What is the Pegasus Topology?](https://support.dwavesys.com/hc/en-us/articles/360054564874-What-Is-the-Pegasus-Topology) for more information
    #[display("Pegasus")]
    #[serde(alias = "pegasus")]
    Pegasus,
    /// D-Wave Chimera graph topology
    ///
    /// See [What is the Chimera Topology?](https://support.dwavesys.com/hc/en-us/articles/360003695354-What-Is-the-Chimera-Topology) for more information
    #[display("Chimera")]
    #[serde(alias = "chimera")]
    Chimera,
    /// Quantinuum Zyng topology
    #[display("Zyng")]
    #[serde(alias = "zyng")]
    Zyng,
    /// Other or unspecified topology
    #[display("{}", _0)]
    Other(String),
}
