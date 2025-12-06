use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

// ============== ROLES & PERMISSIONS ==============

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Role {
    Owner,
    Operator,
    Viewer,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Permission {
    Execute,        // Can request actions
    Configure,      // Can modify settings
    ViewLogs,       // Can read audit logs
    Sign,           // Can sign pending requests
    Emergency,      // Can pause/resume
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct RoleAssignment {
    pub principal: Principal,
    pub role: Role,
    pub assigned_at: u64,
    pub assigned_by: Principal,
}

// ============== POLICIES ==============

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Policy {
    pub name: String,
    pub conditions: Vec<Condition>,
    pub action: PolicyAction,
    pub priority: u32,  // Lower = higher priority
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Condition {
    MaxAmount(u64),                    // Max amount per transaction
    DailyLimit(u64),                   // Max daily volume
    AllowedTokens(Vec<String>),        // Whitelist of token addresses
    AllowedChains(Vec<String>),        // Whitelist of chains
    TimeWindow { start: u64, end: u64 }, // Allowed hours (UTC)
    Cooldown(u64),                     // Seconds between operations
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum PolicyAction {
    Allow,
    Deny,
    RequireThreshold { required: u8, from_roles: Vec<Role> },
}

// ============== ACTIONS ==============

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum Action {
    Swap {
        chain: String,
        token_in: String,
        token_out: String,
        amount_in: u64,
        min_amount_out: u64,
    },
    Transfer {
        chain: String,
        token: String,
        to: String,
        amount: u64,
    },
    ApproveToken {
        chain: String,
        token: String,
        spender: String,
        amount: u64,
    },
}

// ============== THRESHOLD SIGNING ==============

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PendingRequest {
    pub id: u64,
    pub action: Action,
    pub requester: Principal,
    pub created_at: u64,
    pub expires_at: u64,
    pub required_signatures: u8,
    pub collected_signatures: Vec<Signature>,
    pub status: RequestStatus,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Signature {
    pub signer: Principal,
    pub signed_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum RequestStatus {
    Pending,
    Approved,
    Executed,
    Expired,
    Rejected,
}

// ============== AUDIT LOG ==============

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: u64,
    pub action_type: String,
    pub action_params: String,  // JSON serialized
    pub requester: Principal,
    pub policy_result: PolicyResult,
    pub threshold_request_id: Option<u64>,
    pub execution_result: Option<ExecutionResult>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PolicyResult {
    pub decision: PolicyDecision,
    pub matched_policy: Option<String>,
    pub reason: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum PolicyDecision {
    Allowed,
    Denied,
    RequiresThreshold,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub chain: String,
    pub tx_hash: Option<String>,
    pub error: Option<String>,
}

// ============== API RESPONSES ==============

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum ActionResult {
    Executed(ExecutionResult),
    PendingSignatures(PendingRequest),
    Denied { reason: String },
}

// ============== CONFIGURATION ==============

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ChainGuardConfig {
    pub name: String,
    pub default_threshold: ThresholdConfig,
    pub supported_chains: Vec<String>,
    pub policies: Vec<Policy>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ThresholdConfig {
    pub required: u8,
    pub total: u8,
}
