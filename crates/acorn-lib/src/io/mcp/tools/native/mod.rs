//! Native capability leaves share bounded input and artifact policies.
pub use analysis::{
    AnalysisBatchOutput, AnalysisCategoryOutput, AnalysisCategoryStatus, AnalysisFinding, AnalysisFixDifference, AnalysisFixInput, AnalysisFixOutput,
    AnalysisInput, AnalysisOutput,
};
pub use diff::{DiffOperand, DiffOperation, DiffOperationInput, DiffOperationOutput};

mod analysis;
mod diff;
