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

        // Check if already initialized
        if state.config.is_some() {
            return Err("Already initialized".to_string());
        }

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
        state.access_control.get_roles(&principal)
    })
}

#[query]
fn list_role_assignments() -> Vec<(Principal, Role)> {
    STATE.with(|state| {
        let state = state.borrow();
        state.access_control.list_role_assignments()
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

#[update]
fn update_policy(index: u64, policy: Policy) -> Result<(), String> {
    let caller = ic_cdk::caller();

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        if !state.access_control.has_permission(&caller, &Permission::Configure) {
            return Err("No permission to update policies".to_string());
        }

        if state.access_control.update_policy(index as usize, policy) {
            Ok(())
        } else {
            Err("Policy not found".to_string())
        }
    })
}

#[update]
fn remove_policy(index: u64) -> Result<(), String> {
    let caller = ic_cdk::caller();

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        if !state.access_control.has_permission(&caller, &Permission::Configure) {
            return Err("No permission to remove policies".to_string());
        }

        if state.access_control.remove_policy(index as usize) {
            Ok(())
        } else {
            Err("Policy not found".to_string())
        }
    })
}

#[query]
fn list_policies() -> Vec<Policy> {
    STATE.with(|state| {
        let state = state.borrow();
        state.access_control.get_policies()
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
                let required_sigs = state.config.as_ref().unwrap().default_threshold.required;
                let request = state.threshold_signer.create_request(
                    action.clone(),
                    caller,
                    required_sigs,
                    current_time,
                );

                state.audit_log.log_action(&action, caller, policy_result, Some(request.id), current_time);
                ActionResult::PendingSignatures(request)
            }
            PolicyDecision::Allowed => {
                // Log the action
                let audit_id = state.audit_log.log_action(&action, caller, policy_result, None, current_time);

                // TODO: Execute via ic-alloy (Week 3)
                let result = ExecutionResult {
                    success: true,
                    chain: "ethereum".to_string(),
                    tx_hash: Some("0x...mock".to_string()),
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
            // TODO: Execute the action via ic-alloy (Week 3)
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

#[query]
fn get_audit_entry(id: u64) -> Option<AuditEntry> {
    let caller = ic_cdk::caller();

    STATE.with(|state| {
        let state = state.borrow();

        if !state.access_control.has_permission(&caller, &Permission::ViewLogs) {
            return None;
        }

        state.audit_log.get_entry(id).cloned()
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
