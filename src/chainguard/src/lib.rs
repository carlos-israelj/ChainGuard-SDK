use candid::Principal;
use ic_cdk::api::time;
use ic_cdk_macros::{init, query, update};
use std::cell::RefCell;

mod types;
mod access_control;
mod threshold;
mod audit;
mod errors;
mod executor;
mod evm_rpc;
mod btc_rpc;
mod btc_address;
mod btc_signing;
mod btc_transaction;
mod config;
mod abi;
mod universal_router;

use types::*;
use access_control::AccessControl;
use threshold::ThresholdSigner;
use audit::AuditLog;
use executor::ChainExecutor;

thread_local! {
    static STATE: RefCell<ChainGuardState> = RefCell::new(ChainGuardState::default());
}

struct ChainGuardState {
    config: Option<ChainGuardConfig>,
    access_control: AccessControl,
    threshold_signer: ThresholdSigner,
    audit_log: AuditLog,
    executor: ChainExecutor,
    paused: bool,
    daily_volume: u64,
    last_reset: u64,
}

impl Default for ChainGuardState {
    fn default() -> Self {
        Self {
            config: None,
            access_control: AccessControl::default(),
            threshold_signer: ThresholdSigner::default(),
            audit_log: AuditLog::default(),
            executor: ChainExecutor::default(),
            paused: false,
            daily_volume: 0,
            last_reset: 0,
        }
    }
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

    // Evaluate policy and create audit entry
    let (decision, audit_id_opt) = STATE.with(|state| {
        let mut state = state.borrow_mut();

        // Check if paused
        if state.paused {
            return (None, None);
        }

        // Check permission
        if !state.access_control.has_permission(&caller, &Permission::Execute) {
            return (None, None);
        }

        // Evaluate policies
        let policy_result = state.access_control.evaluate_action(&action, &caller, state.daily_volume);

        match policy_result.decision {
            PolicyDecision::Denied => {
                state.audit_log.log_action(&action, caller, policy_result.clone(), None, current_time);
                (Some(PolicyDecision::Denied), None)
            }
            PolicyDecision::RequiresThreshold => {
                let required_sigs = state.config.as_ref().unwrap().default_threshold.required;
                let request = state.threshold_signer.create_request(
                    action.clone(),
                    caller,
                    required_sigs,
                    current_time,
                );
                state.audit_log.log_action(&action, caller, policy_result, Some(request.id), current_time);
                (Some(PolicyDecision::RequiresThreshold), Some(request.id))
            }
            PolicyDecision::Allowed => {
                let audit_id = state.audit_log.log_action(&action, caller, policy_result, None, current_time);
                (Some(PolicyDecision::Allowed), Some(audit_id))
            }
        }
    });

    // Handle paused state
    if decision.is_none() {
        return ActionResult::Denied { reason: "System is paused or no permission".to_string() };
    }

    match decision.unwrap() {
        PolicyDecision::Denied => {
            ActionResult::Denied { reason: "Policy denied".to_string() }
        }
        PolicyDecision::RequiresThreshold => {
            let request = STATE.with(|state| {
                state.borrow().threshold_signer.get_request(audit_id_opt.unwrap()).cloned()
            });
            ActionResult::PendingSignatures(request.unwrap())
        }
        PolicyDecision::Allowed => {
            // Clone executor to avoid borrow issues across await
            let executor = STATE.with(|state| {
                state.borrow().executor.clone()
            });

            // Execute action using ChainExecutor
            let result = executor.execute_action(&action).await;

            // Update audit log with execution result
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                let _ = state.audit_log.update_execution_result(audit_id_opt.unwrap(), result.clone());
            });

            ActionResult::Executed(result)
        }
    }
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

    // Sign the request and check if approved
    let (request_opt, action_opt) = STATE.with(|state| {
        let mut state = state.borrow_mut();

        // Check permission
        if !state.access_control.has_permission(&caller, &Permission::Sign) {
            return (None, None);
        }

        match state.threshold_signer.sign_request(request_id, caller, current_time) {
            Ok(request) => {
                if request.status == RequestStatus::Approved {
                    // Extract action for execution
                    (Some(request.clone()), Some(request.action.clone()))
                } else {
                    (Some(request), None)
                }
            }
            Err(_) => (None, None),
        }
    });

    let request = request_opt.ok_or("Failed to sign request or no permission".to_string())?;

    // If approved, execute the action
    if let Some(action) = action_opt {
        // Clone executor to avoid borrow issues across await
        let executor = STATE.with(|state| {
            state.borrow().executor.clone()
        });

        // Execute action using ChainExecutor
        let execution_result = executor.execute_action(&action).await;

        // Mark as executed and update audit log
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let _ = state.threshold_signer.mark_executed(request_id);

            // Find and update the corresponding audit entry
            // (audit entry was created when threshold request was made)
            if let Some(audit_entry) = state.audit_log.get_entries(None, None)
                .iter()
                .find(|e| e.threshold_request_id == Some(request_id))
            {
                let _ = state.audit_log.update_execution_result(audit_entry.id, execution_result);
            }
        });
    }

    Ok(request)
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

#[update]
async fn get_eth_address() -> Result<String, String> {
    use crate::evm_rpc::EvmRpcExecutor;

    let (key_name, derivation_path) = STATE.with(|state| {
        let s = state.borrow();
        (s.executor.key_name.clone(), s.executor.derivation_path.clone())
    });

    let evm_executor = EvmRpcExecutor::new(key_name, derivation_path)?;
    evm_executor.get_eth_address().await
}

#[update]
async fn get_bitcoin_address(network: String) -> Result<String, String> {
    use crate::btc_signing::get_p2wpkh_address;

    let (key_name, derivation_path) = STATE.with(|state| {
        let s = state.borrow();
        (s.executor.key_name.clone(), s.executor.derivation_path.clone())
    });

    // Map network string to bitcoin::Network
    let btc_network = match network.as_str() {
        "Bitcoin" => bitcoin::Network::Bitcoin,
        "BitcoinTestnet" => bitcoin::Network::Testnet,
        _ => return Err("Unsupported network".to_string()),
    };

    get_p2wpkh_address(key_name, derivation_path, btc_network)
        .await
        .map_err(|e| format!("{:?}", e))
}

// ============== HTTP OUTCALL TRANSFORM ==============

/// Transform function for Blockstream API HTTP responses
/// This is required for HTTP outcalls to work in consensus
#[query]
fn transform_blockstream_response(args: ic_cdk::api::management_canister::http_request::TransformArgs) -> ic_cdk::api::management_canister::http_request::HttpResponse {
    // Simply return the response as-is
    // In production, you might want to strip headers or normalize the response
    ic_cdk::api::management_canister::http_request::HttpResponse {
        status: args.response.status,
        headers: vec![], // Remove headers for consensus
        body: args.response.body,
    }
}

// Export Candid interface
ic_cdk::export_candid!();
