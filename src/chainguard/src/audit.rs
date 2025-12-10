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
            Action::Swap { chain, token_in, token_out, amount_in, min_amount_out, fee_tier } => {
                let fee_tier_str = fee_tier.map_or("null".to_string(), |ft| ft.to_string());
                format!(
                    r#"{{"chain":"{}","token_in":"{}","token_out":"{}","amount_in":{},"min_amount_out":{},"fee_tier":{}}}"#,
                    chain, token_in, token_out, amount_in, min_amount_out, fee_tier_str
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

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    fn mock_principal(id: u8) -> Principal {
        let mut bytes = [0u8; 29];
        bytes[0] = id;
        Principal::from_slice(&bytes)
    }

    fn mock_action() -> Action {
        Action::Transfer {
            chain: "ethereum".to_string(),
            token: "USDC".to_string(),
            to: "0x123".to_string(),
            amount: 1000,
        }
    }

    fn mock_policy_result_allowed() -> PolicyResult {
        PolicyResult {
            decision: PolicyDecision::Allowed,
            matched_policy: Some("Test Policy".to_string()),
            reason: "Allowed by policy".to_string(),
        }
    }

    #[test]
    fn test_log_action() {
        let mut audit = AuditLog::new();
        let principal = mock_principal(1);
        let action = mock_action();
        let policy_result = mock_policy_result_allowed();

        let entry_id = audit.log_action(&action, principal, policy_result, None, 1000);

        assert_eq!(entry_id, 0);
        assert_eq!(audit.entries.len(), 1);

        let entry = audit.get_entry(entry_id).unwrap();
        assert_eq!(entry.id, 0);
        assert_eq!(entry.timestamp, 1000);
        assert_eq!(entry.action_type, "transfer");
        assert_eq!(entry.requester, principal);
        assert!(entry.execution_result.is_none());
    }

    #[test]
    fn test_multiple_log_entries() {
        let mut audit = AuditLog::new();
        let principal = mock_principal(1);
        let action = mock_action();
        let policy_result = mock_policy_result_allowed();

        let id1 = audit.log_action(&action, principal, policy_result.clone(), None, 1000);
        let id2 = audit.log_action(&action, principal, policy_result.clone(), None, 2000);
        let id3 = audit.log_action(&action, principal, policy_result, None, 3000);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
        assert_eq!(audit.entries.len(), 3);
    }

    #[test]
    fn test_update_execution_result() {
        let mut audit = AuditLog::new();
        let principal = mock_principal(1);
        let action = mock_action();
        let policy_result = mock_policy_result_allowed();

        let entry_id = audit.log_action(&action, principal, policy_result, None, 1000);

        let exec_result = ExecutionResult {
            success: true,
            chain: "ethereum".to_string(),
            tx_hash: Some("0xabc123".to_string()),
            error: None,
        };

        let result = audit.update_execution_result(entry_id, exec_result.clone());
        assert!(result.is_ok());

        let entry = audit.get_entry(entry_id).unwrap();
        assert!(entry.execution_result.is_some());
        let stored_result = entry.execution_result.as_ref().unwrap();
        assert_eq!(stored_result.success, true);
        assert_eq!(stored_result.tx_hash, Some("0xabc123".to_string()));
    }

    #[test]
    fn test_update_execution_result_not_found() {
        let mut audit = AuditLog::new();

        let exec_result = ExecutionResult {
            success: true,
            chain: "ethereum".to_string(),
            tx_hash: Some("0xabc123".to_string()),
            error: None,
        };

        let result = audit.update_execution_result(999, exec_result);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Entry not found");
    }

    #[test]
    fn test_get_entries_all() {
        let mut audit = AuditLog::new();
        let principal = mock_principal(1);
        let action = mock_action();
        let policy_result = mock_policy_result_allowed();

        audit.log_action(&action, principal, policy_result.clone(), None, 1000);
        audit.log_action(&action, principal, policy_result.clone(), None, 2000);
        audit.log_action(&action, principal, policy_result, None, 3000);

        let entries = audit.get_entries(None, None);
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_get_entries_with_start() {
        let mut audit = AuditLog::new();
        let principal = mock_principal(1);
        let action = mock_action();
        let policy_result = mock_policy_result_allowed();

        audit.log_action(&action, principal, policy_result.clone(), None, 1000);
        audit.log_action(&action, principal, policy_result.clone(), None, 2000);
        audit.log_action(&action, principal, policy_result, None, 3000);

        let entries = audit.get_entries(Some(2000), None);
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.timestamp >= 2000));
    }

    #[test]
    fn test_get_entries_with_end() {
        let mut audit = AuditLog::new();
        let principal = mock_principal(1);
        let action = mock_action();
        let policy_result = mock_policy_result_allowed();

        audit.log_action(&action, principal, policy_result.clone(), None, 1000);
        audit.log_action(&action, principal, policy_result.clone(), None, 2000);
        audit.log_action(&action, principal, policy_result, None, 3000);

        let entries = audit.get_entries(None, Some(2000));
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.timestamp <= 2000));
    }

    #[test]
    fn test_get_entries_with_range() {
        let mut audit = AuditLog::new();
        let principal = mock_principal(1);
        let action = mock_action();
        let policy_result = mock_policy_result_allowed();

        audit.log_action(&action, principal, policy_result.clone(), None, 1000);
        audit.log_action(&action, principal, policy_result.clone(), None, 2000);
        audit.log_action(&action, principal, policy_result.clone(), None, 3000);
        audit.log_action(&action, principal, policy_result, None, 4000);

        let entries = audit.get_entries(Some(2000), Some(3000));
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.timestamp >= 2000 && e.timestamp <= 3000));
    }

    #[test]
    fn test_get_entry() {
        let mut audit = AuditLog::new();
        let principal = mock_principal(1);
        let action = mock_action();
        let policy_result = mock_policy_result_allowed();

        let entry_id = audit.log_action(&action, principal, policy_result, None, 1000);

        let entry = audit.get_entry(entry_id);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().id, entry_id);
    }

    #[test]
    fn test_get_entry_not_found() {
        let audit = AuditLog::new();
        let entry = audit.get_entry(999);
        assert!(entry.is_none());
    }

    #[test]
    fn test_action_type_string() {
        let swap = Action::Swap {
            chain: "ethereum".to_string(),
            token_in: "USDC".to_string(),
            token_out: "WETH".to_string(),
            amount_in: 1000,
            min_amount_out: 500,
            fee_tier: None,
        };

        let transfer = Action::Transfer {
            chain: "ethereum".to_string(),
            token: "USDC".to_string(),
            to: "0x123".to_string(),
            amount: 1000,
        };

        let approve = Action::ApproveToken {
            chain: "ethereum".to_string(),
            token: "USDC".to_string(),
            spender: "0x456".to_string(),
            amount: 1000,
        };

        assert_eq!(AuditLog::action_type_string(&swap), "swap");
        assert_eq!(AuditLog::action_type_string(&transfer), "transfer");
        assert_eq!(AuditLog::action_type_string(&approve), "approve");
    }

    #[test]
    fn test_action_to_json_transfer() {
        let action = Action::Transfer {
            chain: "ethereum".to_string(),
            token: "USDC".to_string(),
            to: "0x123".to_string(),
            amount: 1000,
        };

        let json = AuditLog::action_to_json(&action);
        assert!(json.contains("ethereum"));
        assert!(json.contains("USDC"));
        assert!(json.contains("0x123"));
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_action_to_json_swap() {
        let action = Action::Swap {
            chain: "ethereum".to_string(),
            token_in: "USDC".to_string(),
            token_out: "WETH".to_string(),
            amount_in: 1000,
            min_amount_out: 500,
            fee_tier: None,
        };

        let json = AuditLog::action_to_json(&action);
        assert!(json.contains("ethereum"));
        assert!(json.contains("USDC"));
        assert!(json.contains("WETH"));
        assert!(json.contains("1000"));
        assert!(json.contains("500"));
    }

    #[test]
    fn test_log_with_threshold_request_id() {
        let mut audit = AuditLog::new();
        let principal = mock_principal(1);
        let action = mock_action();
        let policy_result = PolicyResult {
            decision: PolicyDecision::RequiresThreshold,
            matched_policy: Some("Threshold Policy".to_string()),
            reason: "Requires 2 signatures".to_string(),
        };

        let entry_id = audit.log_action(&action, principal, policy_result, Some(42), 1000);

        let entry = audit.get_entry(entry_id).unwrap();
        assert_eq!(entry.threshold_request_id, Some(42));
    }
}
