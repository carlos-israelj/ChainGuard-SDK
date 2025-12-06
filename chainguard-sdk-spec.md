# ChainGuard SDK - Technical Specification

## Project Overview

**Name:** ChainGuard SDK
**Type:** Security middleware for AI agents on ICP
**Bounty:** Secure Multi-Chain AI Agents with ICP (Zo House Hackathon)
**Prize:** 500 USDC / 63 ICP tokens
**Deadline:** 27 days

### One-liner
SDK de seguridad que permite a cualquier AI agent ejecutar transacciones multi-chain con access control granular, threshold signatures y auditabilidad completa.

### Bounty Requirements Mapping

| Requirement | How ChainGuard Fulfills It |
|-------------|---------------------------|
| "AI agent or plugin" | Plugin/SDK for any AI agent |
| Access control rules | Core feature - roles, permissions, policies |
| Threshold cryptography | N-of-M approval using ICP Chain-Key |
| Autonomous trading | Enables secure autonomous execution |
| Auditable framework | Complete on-chain audit trail |
| Multi-chain | ETH/EVM via ic-alloy, BTC via ckBTC |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              EXTERNAL AI AGENT                               │
│         (trading bot, DeFi manager, etc)                    │
└─────────────────────────┬───────────────────────────────────┘
                          │ request_action()
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  CHAINGUARD CANISTER                        │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │                 REQUEST HANDLER                        │ │
│  │  - Validate incoming requests                         │ │
│  │  - Route to appropriate module                        │ │
│  └───────────────────────────┬───────────────────────────┘ │
│                              ▼                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │               ACCESS CONTROL MODULE                    │ │
│  │                                                        │ │
│  │  Roles:           Permissions:       Policies:         │ │
│  │  ┌─────────┐     ┌─────────────┐   ┌──────────────┐   │ │
│  │  │ Owner   │     │ execute     │   │ max_amount   │   │ │
│  │  │ Operator│     │ configure   │   │ daily_limit  │   │ │
│  │  │ Viewer  │     │ view_logs   │   │ allowed_tokens│  │ │
│  │  └─────────┘     └─────────────┘   └──────────────┘   │ │
│  │                                                        │ │
│  │  Decision: ALLOW | DENY | REQUIRE_THRESHOLD            │ │
│  └───────────────────────────┬───────────────────────────┘ │
│                              ▼                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │              THRESHOLD SIGNER MODULE                   │ │
│  │                                                        │ │
│  │  PendingRequest {                                      │ │
│  │    id, action, required_sigs, collected_sigs,         │ │
│  │    created_at, expires_at, status                     │ │
│  │  }                                                     │ │
│  │                                                        │ │
│  │  On threshold met → Execute via Chain-Key              │ │
│  └───────────────────────────┬───────────────────────────┘ │
│                              ▼                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │               MULTI-CHAIN EXECUTOR                     │ │
│  │                                                        │ │
│  │  ic-alloy: ETH, EVM chains                            │ │
│  │  ckBTC: Bitcoin (future)                              │ │
│  └───────────────────────────┬───────────────────────────┘ │
│                              ▼                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │                  AUDIT LOG MODULE                      │ │
│  │                                                        │ │
│  │  AuditEntry {                                          │ │
│  │    timestamp, action, requester, policy_result,       │ │
│  │    signatures, execution_result                       │ │
│  │  }                                                     │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Framework | ICP Canister (ic-cdk) |
| Multi-chain | ic-alloy for EVM |
| Signatures | ICP Threshold ECDSA (Chain-Key) |
| Frontend (demo) | React + Vite (optional) |
| Testing | PocketIC |

---

## Project Structure

```
chainguard-sdk/
├── Cargo.toml
├── dfx.json
├── README.md
├── scripts/
│   ├── deploy.sh
│   ├── test.sh
│   └── demo.sh
├── src/
│   └── chainguard/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs              # Main entry point
│       │   ├── types.rs            # All type definitions
│       │   ├── access_control.rs   # Access control module
│       │   ├── threshold.rs        # Threshold signer module
│       │   ├── executor.rs         # Multi-chain executor
│       │   ├── audit.rs            # Audit log module
│       │   └── errors.rs           # Error types
│       └── chainguard.did          # Candid interface
├── demo-agent/
│   └── src/
│       └── main.rs                 # Demo AI agent using ChainGuard
└── tests/
    └── integration_tests.rs
```

---

## Type Definitions (src/chainguard/src/types.rs)

```rust
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
```

---

## API Interface (src/chainguard/chainguard.did)

```candid
type Role = variant {
    Owner;
    Operator;
    Viewer;
};

type Permission = variant {
    Execute;
    Configure;
    ViewLogs;
    Sign;
    Emergency;
};

type Action = variant {
    Swap : record {
        chain : text;
        token_in : text;
        token_out : text;
        amount_in : nat64;
        min_amount_out : nat64;
    };
    Transfer : record {
        chain : text;
        token : text;
        to : text;
        amount : nat64;
    };
    ApproveToken : record {
        chain : text;
        token : text;
        spender : text;
        amount : nat64;
    };
};

type ActionResult = variant {
    Executed : ExecutionResult;
    PendingSignatures : PendingRequest;
    Denied : record { reason : text };
};

type ExecutionResult = record {
    success : bool;
    chain : text;
    tx_hash : opt text;
    error : opt text;
};

type PendingRequest = record {
    id : nat64;
    action : Action;
    requester : principal;
    created_at : nat64;
    expires_at : nat64;
    required_signatures : nat8;
    collected_signatures : vec Signature;
    status : RequestStatus;
};

type Signature = record {
    signer : principal;
    signed_at : nat64;
};

type RequestStatus = variant {
    Pending;
    Approved;
    Executed;
    Expired;
    Rejected;
};

type AuditEntry = record {
    id : nat64;
    timestamp : nat64;
    action_type : text;
    action_params : text;
    requester : principal;
    policy_result : PolicyResult;
    threshold_request_id : opt nat64;
    execution_result : opt ExecutionResult;
};

type PolicyResult = record {
    decision : PolicyDecision;
    matched_policy : opt text;
    reason : text;
};

type PolicyDecision = variant {
    Allowed;
    Denied;
    RequiresThreshold;
};

type Policy = record {
    name : text;
    conditions : vec Condition;
    action : PolicyAction;
    priority : nat32;
};

type Condition = variant {
    MaxAmount : nat64;
    DailyLimit : nat64;
    AllowedTokens : vec text;
    AllowedChains : vec text;
    TimeWindow : record { start : nat64; end : nat64 };
    Cooldown : nat64;
};

type PolicyAction = variant {
    Allow;
    Deny;
    RequireThreshold : record { required : nat8; from_roles : vec Role };
};

type ChainGuardConfig = record {
    name : text;
    default_threshold : record { required : nat8; total : nat8 };
    supported_chains : vec text;
    policies : vec Policy;
};

service : {
    // Initialization
    initialize : (ChainGuardConfig) -> ();
    
    // Role Management
    assign_role : (principal, Role) -> (variant { Ok; Err : text });
    revoke_role : (principal, Role) -> (variant { Ok; Err : text });
    get_roles : (principal) -> (vec Role) query;
    list_role_assignments : () -> (vec record { principal; Role }) query;
    
    // Policy Management
    add_policy : (Policy) -> (variant { Ok : nat64; Err : text });
    update_policy : (nat64, Policy) -> (variant { Ok; Err : text });
    remove_policy : (nat64) -> (variant { Ok; Err : text });
    list_policies : () -> (vec Policy) query;
    
    // Action Execution (for AI agents)
    request_action : (Action) -> (ActionResult);
    
    // Threshold Signing (for signers)
    get_pending_requests : () -> (vec PendingRequest) query;
    sign_request : (nat64) -> (variant { Ok : PendingRequest; Err : text });
    reject_request : (nat64, text) -> (variant { Ok; Err : text });
    
    // Audit
    get_audit_logs : (opt nat64, opt nat64) -> (vec AuditEntry) query;
    get_audit_entry : (nat64) -> (opt AuditEntry) query;
    
    // Emergency
    pause : () -> (variant { Ok; Err : text });
    resume : () -> (variant { Ok; Err : text });
    is_paused : () -> (bool) query;
    
    // Info
    get_config : () -> (ChainGuardConfig) query;
    get_eth_address : () -> (text);
}
```

---

## Module Implementation Guide

### 1. Access Control Module (src/chainguard/src/access_control.rs)

```rust
use crate::types::*;
use candid::Principal;
use std::collections::HashMap;

pub struct AccessControl {
    role_assignments: HashMap<Principal, Vec<Role>>,
    policies: Vec<Policy>,
}

impl AccessControl {
    pub fn new() -> Self {
        Self {
            role_assignments: HashMap::new(),
            policies: Vec::new(),
        }
    }
    
    // Check if principal has a specific role
    pub fn has_role(&self, principal: &Principal, role: &Role) -> bool {
        self.role_assignments
            .get(principal)
            .map(|roles| roles.contains(role))
            .unwrap_or(false)
    }
    
    // Check if principal has permission (derived from roles)
    pub fn has_permission(&self, principal: &Principal, permission: &Permission) -> bool {
        let roles = self.role_assignments.get(principal);
        match roles {
            None => false,
            Some(roles) => {
                for role in roles {
                    if Self::role_has_permission(role, permission) {
                        return true;
                    }
                }
                false
            }
        }
    }
    
    // Define which roles have which permissions
    fn role_has_permission(role: &Role, permission: &Permission) -> bool {
        match (role, permission) {
            (Role::Owner, _) => true,  // Owner has all permissions
            (Role::Operator, Permission::Execute) => true,
            (Role::Operator, Permission::Sign) => true,
            (Role::Operator, Permission::ViewLogs) => true,
            (Role::Viewer, Permission::ViewLogs) => true,
            _ => false,
        }
    }
    
    // Evaluate policies for an action
    pub fn evaluate_action(&self, action: &Action, requester: &Principal, daily_spent: u64) -> PolicyResult {
        // Sort policies by priority
        let mut sorted_policies = self.policies.clone();
        sorted_policies.sort_by_key(|p| p.priority);
        
        for policy in &sorted_policies {
            if self.conditions_match(&policy.conditions, action, daily_spent) {
                return PolicyResult {
                    decision: self.policy_action_to_decision(&policy.action),
                    matched_policy: Some(policy.name.clone()),
                    reason: format!("Matched policy: {}", policy.name),
                };
            }
        }
        
        // Default: deny if no policy matches
        PolicyResult {
            decision: PolicyDecision::Denied,
            matched_policy: None,
            reason: "No matching policy found".to_string(),
        }
    }
    
    fn conditions_match(&self, conditions: &[Condition], action: &Action, daily_spent: u64) -> bool {
        let amount = self.get_action_amount(action);
        
        for condition in conditions {
            match condition {
                Condition::MaxAmount(max) => {
                    if amount > *max {
                        return false;
                    }
                }
                Condition::DailyLimit(limit) => {
                    if daily_spent + amount > *limit {
                        return false;
                    }
                }
                Condition::AllowedChains(chains) => {
                    let chain = self.get_action_chain(action);
                    if !chains.contains(&chain) {
                        return false;
                    }
                }
                // Add more condition checks...
                _ => {}
            }
        }
        true
    }
    
    fn get_action_amount(&self, action: &Action) -> u64 {
        match action {
            Action::Swap { amount_in, .. } => *amount_in,
            Action::Transfer { amount, .. } => *amount,
            Action::ApproveToken { amount, .. } => *amount,
        }
    }
    
    fn get_action_chain(&self, action: &Action) -> String {
        match action {
            Action::Swap { chain, .. } => chain.clone(),
            Action::Transfer { chain, .. } => chain.clone(),
            Action::ApproveToken { chain, .. } => chain.clone(),
        }
    }
    
    fn policy_action_to_decision(&self, action: &PolicyAction) -> PolicyDecision {
        match action {
            PolicyAction::Allow => PolicyDecision::Allowed,
            PolicyAction::Deny => PolicyDecision::Denied,
            PolicyAction::RequireThreshold { .. } => PolicyDecision::RequiresThreshold,
        }
    }
    
    // CRUD operations for roles
    pub fn assign_role(&mut self, principal: Principal, role: Role) {
        self.role_assignments
            .entry(principal)
            .or_insert_with(Vec::new)
            .push(role);
    }
    
    pub fn revoke_role(&mut self, principal: &Principal, role: &Role) {
        if let Some(roles) = self.role_assignments.get_mut(principal) {
            roles.retain(|r| r != role);
        }
    }
    
    // CRUD operations for policies
    pub fn add_policy(&mut self, policy: Policy) -> u64 {
        let id = self.policies.len() as u64;
        self.policies.push(policy);
        id
    }
    
    pub fn remove_policy(&mut self, index: usize) -> bool {
        if index < self.policies.len() {
            self.policies.remove(index);
            true
        } else {
            false
        }
    }
}
```

### 2. Threshold Module (src/chainguard/src/threshold.rs)

```rust
use crate::types::*;
use candid::Principal;
use std::collections::HashMap;

pub struct ThresholdSigner {
    pending_requests: HashMap<u64, PendingRequest>,
    next_id: u64,
    default_expiry: u64,  // seconds
}

impl ThresholdSigner {
    pub fn new() -> Self {
        Self {
            pending_requests: HashMap::new(),
            next_id: 0,
            default_expiry: 86400,  // 24 hours
        }
    }
    
    pub fn create_request(
        &mut self,
        action: Action,
        requester: Principal,
        required_signatures: u8,
        current_time: u64,
    ) -> PendingRequest {
        let id = self.next_id;
        self.next_id += 1;
        
        let request = PendingRequest {
            id,
            action,
            requester,
            created_at: current_time,
            expires_at: current_time + self.default_expiry,
            required_signatures,
            collected_signatures: Vec::new(),
            status: RequestStatus::Pending,
        };
        
        self.pending_requests.insert(id, request.clone());
        request
    }
    
    pub fn sign_request(
        &mut self,
        request_id: u64,
        signer: Principal,
        current_time: u64,
    ) -> Result<PendingRequest, String> {
        let request = self.pending_requests
            .get_mut(&request_id)
            .ok_or("Request not found")?;
        
        // Check if expired
        if current_time > request.expires_at {
            request.status = RequestStatus::Expired;
            return Err("Request expired".to_string());
        }
        
        // Check if already signed by this signer
        if request.collected_signatures.iter().any(|s| s.signer == signer) {
            return Err("Already signed by this principal".to_string());
        }
        
        // Check status
        if request.status != RequestStatus::Pending {
            return Err(format!("Request is not pending, status: {:?}", request.status));
        }
        
        // Add signature
        request.collected_signatures.push(Signature {
            signer,
            signed_at: current_time,
        });
        
        // Check if threshold reached
        if request.collected_signatures.len() >= request.required_signatures as usize {
            request.status = RequestStatus::Approved;
        }
        
        Ok(request.clone())
    }
    
    pub fn reject_request(
        &mut self,
        request_id: u64,
        _reason: String,
    ) -> Result<(), String> {
        let request = self.pending_requests
            .get_mut(&request_id)
            .ok_or("Request not found")?;
        
        request.status = RequestStatus::Rejected;
        Ok(())
    }
    
    pub fn mark_executed(&mut self, request_id: u64) -> Result<(), String> {
        let request = self.pending_requests
            .get_mut(&request_id)
            .ok_or("Request not found")?;
        
        request.status = RequestStatus::Executed;
        Ok(())
    }
    
    pub fn get_pending_requests(&self) -> Vec<PendingRequest> {
        self.pending_requests
            .values()
            .filter(|r| r.status == RequestStatus::Pending)
            .cloned()
            .collect()
    }
    
    pub fn get_request(&self, id: u64) -> Option<&PendingRequest> {
        self.pending_requests.get(&id)
    }
    
    pub fn is_approved(&self, request_id: u64) -> bool {
        self.pending_requests
            .get(&request_id)
            .map(|r| r.status == RequestStatus::Approved)
            .unwrap_or(false)
    }
    
    // Cleanup expired requests
    pub fn cleanup_expired(&mut self, current_time: u64) {
        for request in self.pending_requests.values_mut() {
            if request.status == RequestStatus::Pending && current_time > request.expires_at {
                request.status = RequestStatus::Expired;
            }
        }
    }
}
```

### 3. Audit Module (src/chainguard/src/audit.rs)

```rust
use crate::types::*;
use candid::Principal;

pub struct AuditLog {
    entries: Vec<AuditEntry>,
    next_id: u64,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
        }
    }
    
    pub fn log_action(
        &mut self,
        action: &Action,
        requester: Principal,
        policy_result: PolicyResult,
        threshold_request_id: Option<u64>,
        current_time: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        
        let entry = AuditEntry {
            id,
            timestamp: current_time,
            action_type: Self::action_type_string(action),
            action_params: Self::action_to_json(action),
            requester,
            policy_result,
            threshold_request_id,
            execution_result: None,
        };
        
        self.entries.push(entry);
        id
    }
    
    pub fn update_execution_result(
        &mut self,
        entry_id: u64,
        result: ExecutionResult,
    ) -> Result<(), String> {
        let entry = self.entries
            .iter_mut()
            .find(|e| e.id == entry_id)
            .ok_or("Entry not found")?;
        
        entry.execution_result = Some(result);
        Ok(())
    }
    
    pub fn get_entries(&self, start: Option<u64>, end: Option<u64>) -> Vec<AuditEntry> {
        self.entries
            .iter()
            .filter(|e| {
                let after_start = start.map(|s| e.timestamp >= s).unwrap_or(true);
                let before_end = end.map(|e_time| e.timestamp <= e_time).unwrap_or(true);
                after_start && before_end
            })
            .cloned()
            .collect()
    }
    
    pub fn get_entry(&self, id: u64) -> Option<&AuditEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
    
    fn action_type_string(action: &Action) -> String {
        match action {
            Action::Swap { .. } => "swap".to_string(),
            Action::Transfer { .. } => "transfer".to_string(),
            Action::ApproveToken { .. } => "approve".to_string(),
        }
    }
    
    fn action_to_json(action: &Action) -> String {
        // Simple JSON serialization
        match action {
            Action::Swap { chain, token_in, token_out, amount_in, min_amount_out } => {
                format!(
                    r#"{{"chain":"{}","token_in":"{}","token_out":"{}","amount_in":{},"min_amount_out":{}}}"#,
                    chain, token_in, token_out, amount_in, min_amount_out
                )
            }
            Action::Transfer { chain, token, to, amount } => {
                format!(
                    r#"{{"chain":"{}","token":"{}","to":"{}","amount":{}}}"#,
                    chain, token, to, amount
                )
            }
            Action::ApproveToken { chain, token, spender, amount } => {
                format!(
                    r#"{{"chain":"{}","token":"{}","spender":"{}","amount":{}}}"#,
                    chain, token, spender, amount
                )
            }
        }
    }
}
```

### 4. Main Entry Point (src/chainguard/src/lib.rs)

```rust
use candid::Principal;
use ic_cdk::api::time;
use ic_cdk_macros::{init, query, update};
use std::cell::RefCell;

mod types;
mod access_control;
mod threshold;
mod audit;
mod errors;

use types::*;
use access_control::AccessControl;
use threshold::ThresholdSigner;
use audit::AuditLog;

thread_local! {
    static STATE: RefCell<ChainGuardState> = RefCell::new(ChainGuardState::default());
}

#[derive(Default)]
struct ChainGuardState {
    config: Option<ChainGuardConfig>,
    access_control: AccessControl,
    threshold_signer: ThresholdSigner,
    audit_log: AuditLog,
    paused: bool,
    daily_volume: u64,
    last_reset: u64,
}

// ============== INITIALIZATION ==============

#[init]
fn init() {
    // Set deployer as owner
    let caller = ic_cdk::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.access_control.assign_role(caller, Role::Owner);
    });
}

#[update]
fn initialize(config: ChainGuardConfig) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        // Only owner can initialize
        if !state.access_control.has_role(&caller, &Role::Owner) {
            return Err("Only owner can initialize".to_string());
        }
        
        // Add policies from config
        for policy in &config.policies {
            state.access_control.add_policy(policy.clone());
        }
        
        state.config = Some(config);
        Ok(())
    })
}

// ============== ROLE MANAGEMENT ==============

#[update]
fn assign_role(principal: Principal, role: Role) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if !state.access_control.has_permission(&caller, &Permission::Configure) {
            return Err("No permission to assign roles".to_string());
        }
        
        state.access_control.assign_role(principal, role);
        Ok(())
    })
}

#[update]
fn revoke_role(principal: Principal, role: Role) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if !state.access_control.has_permission(&caller, &Permission::Configure) {
            return Err("No permission to revoke roles".to_string());
        }
        
        state.access_control.revoke_role(&principal, &role);
        Ok(())
    })
}

#[query]
fn get_roles(principal: Principal) -> Vec<Role> {
    STATE.with(|state| {
        let state = state.borrow();
        // Return roles for principal
        vec![]  // TODO: implement
    })
}

// ============== POLICY MANAGEMENT ==============

#[update]
fn add_policy(policy: Policy) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if !state.access_control.has_permission(&caller, &Permission::Configure) {
            return Err("No permission to add policies".to_string());
        }
        
        Ok(state.access_control.add_policy(policy))
    })
}

#[query]
fn list_policies() -> Vec<Policy> {
    STATE.with(|state| {
        let state = state.borrow();
        vec![]  // TODO: return policies
    })
}

// ============== ACTION EXECUTION ==============

#[update]
async fn request_action(action: Action) -> ActionResult {
    let caller = ic_cdk::caller();
    let current_time = time();
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        // Check if paused
        if state.paused {
            return ActionResult::Denied { reason: "System is paused".to_string() };
        }
        
        // Check permission
        if !state.access_control.has_permission(&caller, &Permission::Execute) {
            return ActionResult::Denied { reason: "No execute permission".to_string() };
        }
        
        // Evaluate policies
        let policy_result = state.access_control.evaluate_action(&action, &caller, state.daily_volume);
        
        match policy_result.decision {
            PolicyDecision::Denied => {
                // Log and deny
                state.audit_log.log_action(&action, caller, policy_result.clone(), None, current_time);
                ActionResult::Denied { reason: policy_result.reason }
            }
            PolicyDecision::RequiresThreshold => {
                // Create pending request
                let config = state.config.as_ref().unwrap();
                let request = state.threshold_signer.create_request(
                    action.clone(),
                    caller,
                    config.default_threshold.required,
                    current_time,
                );
                
                state.audit_log.log_action(&action, caller, policy_result, Some(request.id), current_time);
                ActionResult::PendingSignatures(request)
            }
            PolicyDecision::Allowed => {
                // Log the action
                let audit_id = state.audit_log.log_action(&action, caller, policy_result, None, current_time);
                
                // TODO: Execute via ic-alloy
                let result = ExecutionResult {
                    success: true,
                    chain: "ethereum".to_string(),
                    tx_hash: Some("0x...".to_string()),
                    error: None,
                };
                
                let _ = state.audit_log.update_execution_result(audit_id, result.clone());
                ActionResult::Executed(result)
            }
        }
    })
}

// ============== THRESHOLD SIGNING ==============

#[query]
fn get_pending_requests() -> Vec<PendingRequest> {
    STATE.with(|state| {
        state.borrow().threshold_signer.get_pending_requests()
    })
}

#[update]
async fn sign_request(request_id: u64) -> Result<PendingRequest, String> {
    let caller = ic_cdk::caller();
    let current_time = time();
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        // Check permission
        if !state.access_control.has_permission(&caller, &Permission::Sign) {
            return Err("No sign permission".to_string());
        }
        
        let request = state.threshold_signer.sign_request(request_id, caller, current_time)?;
        
        // If approved, execute
        if request.status == RequestStatus::Approved {
            // TODO: Execute the action via ic-alloy
            state.threshold_signer.mark_executed(request_id)?;
        }
        
        Ok(request)
    })
}

#[update]
fn reject_request(request_id: u64, reason: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if !state.access_control.has_permission(&caller, &Permission::Sign) {
            return Err("No sign permission".to_string());
        }
        
        state.threshold_signer.reject_request(request_id, reason)
    })
}

// ============== AUDIT ==============

#[query]
fn get_audit_logs(start: Option<u64>, end: Option<u64>) -> Vec<AuditEntry> {
    let caller = ic_cdk::caller();
    
    STATE.with(|state| {
        let state = state.borrow();
        
        if !state.access_control.has_permission(&caller, &Permission::ViewLogs) {
            return vec![];
        }
        
        state.audit_log.get_entries(start, end)
    })
}

// ============== EMERGENCY ==============

#[update]
fn pause() -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if !state.access_control.has_permission(&caller, &Permission::Emergency) {
            return Err("No emergency permission".to_string());
        }
        
        state.paused = true;
        Ok(())
    })
}

#[update]
fn resume() -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if !state.access_control.has_permission(&caller, &Permission::Emergency) {
            return Err("No emergency permission".to_string());
        }
        
        state.paused = false;
        Ok(())
    })
}

#[query]
fn is_paused() -> bool {
    STATE.with(|state| state.borrow().paused)
}

// ============== INFO ==============

#[query]
fn get_config() -> Option<ChainGuardConfig> {
    STATE.with(|state| state.borrow().config.clone())
}

// Export Candid interface
ic_cdk::export_candid!();
```

---

## Setup Commands

```bash
# 1. Install dfx (ICP SDK)
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. Add wasm target
rustup target add wasm32-unknown-unknown

# 4. Create project
mkdir chainguard-sdk && cd chainguard-sdk

# 5. Initialize dfx project
dfx new chainguard --type rust --no-frontend

# 6. Start local replica
dfx start --background

# 7. Deploy
dfx deploy
```

---

## dfx.json Configuration

```json
{
  "canisters": {
    "chainguard": {
      "type": "rust",
      "candid": "src/chainguard/chainguard.did",
      "package": "chainguard"
    }
  },
  "defaults": {
    "build": {
      "packtool": ""
    }
  },
  "version": 1
}
```

---

## Cargo.toml (root)

```toml
[workspace]
members = [
    "src/chainguard",
]
resolver = "2"
```

---

## Cargo.toml (src/chainguard/)

```toml
[package]
name = "chainguard"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
candid = "0.10"
ic-cdk = "0.13"
ic-cdk-macros = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# For ic-alloy integration (add later)
# alloy = { git = "https://github.com/ic-alloy/ic-alloy.git", tag = "v0.3.5-icp.1", features = ["icp"] }
# getrandom = { version = "0.2.15", features = ["custom"] }
```

---

## Development Timeline

### Week 1 (Days 1-7): Foundation
- [ ] Day 1-2: Setup project structure, dfx config
- [ ] Day 3-4: Implement types.rs and access_control.rs
- [ ] Day 5-6: Implement threshold.rs
- [ ] Day 7: Basic tests, fix compilation errors

### Week 2 (Days 8-14): Core Features
- [ ] Day 8-9: Implement audit.rs
- [ ] Day 10-11: Implement lib.rs (main canister)
- [ ] Day 12-14: Integration tests, debug

### Week 3 (Days 15-21): Multi-chain & Demo
- [ ] Day 15-17: Add ic-alloy integration for ETH
- [ ] Day 18-19: Create demo agent
- [ ] Day 20-21: Test on testnet

### Week 4 (Days 22-27): Polish & Submit
- [ ] Day 22-23: Documentation (README, examples)
- [ ] Day 24-25: Video demo
- [ ] Day 26-27: Final testing, submit to hackathon

---

## Testing Commands

```bash
# Run local tests
cargo test

# Deploy locally
dfx start --background
dfx deploy

# Call canister
dfx canister call chainguard get_config

# Assign role
dfx canister call chainguard assign_role '(principal "...", variant { Operator })'

# Request action
dfx canister call chainguard request_action '(variant { Swap = record { chain = "ethereum"; token_in = "USDC"; token_out = "WETH"; amount_in = 100; min_amount_out = 0 } })'
```

---

## Success Criteria (Bounty)

- [x] Fully functional AI agent deployed on ICP
- [x] Open-source code submission
- [x] Access control rules for onchain AI agents
- [x] Threshold cryptography for secure signing
- [x] Enable AI agents to execute trades autonomously
- [x] Transparent, auditable AI agent framework

---

## Notes for Claude Code

When implementing:

1. **Start with types.rs** - Get all types compiling first
2. **Test each module independently** before integrating
3. **Use `ic_cdk::api::time()`** for timestamps
4. **Use `ic_cdk::caller()`** to get caller principal
5. **The `thread_local!` pattern** is required for state in ICP canisters
6. **ic-alloy integration** can be added in Week 3 - focus on core logic first
7. **Keep the Candid file (.did)** in sync with Rust code

---

## Resources

- [ICP Rust CDK Docs](https://docs.rs/ic-cdk/latest/ic_cdk/)
- [ic-alloy Documentation](https://o7kje-7yaaa-aaaal-qnaua-cai.icp0.io/getting-started.html)
- [ic-alloy-dca Example](https://github.com/ic-alloy/ic-alloy-dca)
- [Threshold ECDSA on ICP](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-ecdsa)
