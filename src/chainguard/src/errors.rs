use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum ChainGuardError {
    // Permission errors
    Unauthorized,
    InsufficientPermissions { required: String },

    // Configuration errors
    NotInitialized,
    AlreadyInitialized,
    InvalidConfiguration { reason: String },

    // Policy errors
    PolicyNotFound { id: u64 },
    PolicyEvaluationFailed { reason: String },

    // Threshold errors
    RequestNotFound { id: u64 },
    RequestExpired,
    RequestAlreadySigned,
    RequestNotApproved,
    InvalidRequestStatus { expected: String, actual: String },

    // Execution errors
    ExecutionFailed { reason: String },
    ChainNotSupported { chain: String },
    InsufficientFunds { msg: String },
    InvalidInput { msg: String },
    UnsupportedChain { msg: String },
    NotImplemented { feature: String },

    // System errors
    SystemPaused,
    InternalError { msg: String },
}

impl ChainGuardError {
    pub fn to_string(&self) -> String {
        match self {
            ChainGuardError::Unauthorized => "Unauthorized access".to_string(),
            ChainGuardError::InsufficientPermissions { required } => {
                format!("Insufficient permissions. Required: {}", required)
            }
            ChainGuardError::NotInitialized => "ChainGuard not initialized".to_string(),
            ChainGuardError::AlreadyInitialized => "ChainGuard already initialized".to_string(),
            ChainGuardError::InvalidConfiguration { reason } => {
                format!("Invalid configuration: {}", reason)
            }
            ChainGuardError::PolicyNotFound { id } => format!("Policy not found: {}", id),
            ChainGuardError::PolicyEvaluationFailed { reason } => {
                format!("Policy evaluation failed: {}", reason)
            }
            ChainGuardError::RequestNotFound { id } => format!("Request not found: {}", id),
            ChainGuardError::RequestExpired => "Request has expired".to_string(),
            ChainGuardError::RequestAlreadySigned => {
                "Request already signed by this principal".to_string()
            }
            ChainGuardError::RequestNotApproved => "Request not yet approved".to_string(),
            ChainGuardError::InvalidRequestStatus { expected, actual } => {
                format!("Invalid request status. Expected: {}, Actual: {}", expected, actual)
            }
            ChainGuardError::ExecutionFailed { reason } => {
                format!("Execution failed: {}", reason)
            }
            ChainGuardError::ChainNotSupported { chain } => {
                format!("Chain not supported: {}", chain)
            }
            ChainGuardError::InsufficientFunds { msg } => {
                format!("Insufficient funds: {}", msg)
            }
            ChainGuardError::InvalidInput { msg } => {
                format!("Invalid input: {}", msg)
            }
            ChainGuardError::UnsupportedChain { msg } => {
                format!("Unsupported chain: {}", msg)
            }
            ChainGuardError::NotImplemented { feature } => {
                format!("Feature not yet implemented: {}", feature)
            }
            ChainGuardError::SystemPaused => "System is currently paused".to_string(),
            ChainGuardError::InternalError { msg } => format!("Internal error: {}", msg),
        }
    }
}

pub type ChainGuardResult<T> = Result<T, ChainGuardError>;
