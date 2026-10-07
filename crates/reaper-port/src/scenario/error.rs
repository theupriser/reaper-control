use thiserror::Error;

/// Why a scenario could not be loaded or did not pass.
#[derive(Debug, Error)]
pub enum ScenarioError {
    /// The file is not valid scenario JSON.
    #[error("scenario file is not valid: {0}")]
    Parse(#[from] serde_json::Error),
    /// The scenario describes something impossible.
    #[error("scenario is invalid: {0}")]
    Invalid(String),
    /// An expectation did not hold.
    #[error("step {step} failed: {message}")]
    Failed {
        /// 1-based number of the step.
        step: usize,
        /// What was expected and what was seen.
        message: String,
    },
}
