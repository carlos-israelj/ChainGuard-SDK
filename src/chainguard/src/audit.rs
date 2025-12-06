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

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}
