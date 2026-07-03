//! Quantum computing paradigms and hardware definitions
use crate::prelude::*;
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
/// Quantum computing paradigm or modality
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum Paradigm {
    /// Gate-based quantum computing (e.g., IBM, Google)
    #[display("Gate-based")]
    #[serde(alias = "gate-based", alias = "gate_based", alias = "GateBased")]
    GateBased,
    /// Quantum annealing (e.g., D-Wave)
    #[display("Annealing")]
    Annealing,
    /// Hybrid quantum-classical workflows
    #[display("Hybrid Quantum-Classical")]
    #[serde(
        alias = "Hybrid Quantum-Classical",
        alias = "hybrid quantum-classical",
        alias = "hybrid quantum classical",
        alias = "hybrid_quantum_classical",
        alias = "HybridQuantumClassical"
    )]
    HybridQuantumClassical,
    /// Photonic quantum computing (e.g., PsiQuantum, Xanadu)
    #[display("Photonic")]
    Photonic,
    /// Neutral atom quantum computing (e.g., QuEra, Pasqal)
    #[display("Neutral Atom")]
    #[serde(alias = "Neutral Atom", alias = "neutral_atom", alias = "NeutralAtom")]
    NeutralAtom,
    /// Noisy Intermediate-Scale Quantum (NISQ) computing
    #[display("NISQ")]
    #[serde(
        alias = "NISQ",
        alias = "nisq",
        alias = "Noisy Intermediate-Scale Quantum",
        alias = "noisy intermediate-scale quantum",
        alias = "noisy_intermediate_scale_quantum",
        alias = "NoisyIntermediateScaleQuantum"
    )]
    Nisq,
    /// Trapped ion quantum computing (e.g., IonQ, Honeywell)
    #[display("Trapped Ion")]
    #[serde(alias = "Trapped Ion", alias = "trapped_ion", alias = "TrappedIon")]
    TrappedIon,
    /// Other or unspecified quantum paradigm
    #[display("{}", _0)]
    Other(String),
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
/// [1] J. Kitching et al., "Independent Panel Report for Technical Assessment of NASA and External Quantum Sensing Capabilities," 2023.
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
