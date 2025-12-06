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

    // Get all roles for a principal
    pub fn get_roles(&self, principal: &Principal) -> Vec<Role> {
        self.role_assignments
            .get(principal)
            .cloned()
            .unwrap_or_default()
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
    pub fn evaluate_action(&self, action: &Action, _requester: &Principal, daily_spent: u64) -> PolicyResult {
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
        let chain = self.get_action_chain(action);

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
                    if !chains.contains(&chain) {
                        return false;
                    }
                }
                Condition::AllowedTokens(tokens) => {
                    let action_tokens = self.get_action_tokens(action);
                    for token in &action_tokens {
                        if !tokens.contains(token) {
                            return false;
                        }
                    }
                }
                Condition::TimeWindow { start, end } => {
                    // For now, we'll skip time window checks
                    // In production, would compare current time with start/end
                    let _current_hour = 0; // TODO: implement time check
                    if *start > *end {
                        return false;
                    }
                }
                Condition::Cooldown(_seconds) => {
                    // TODO: implement cooldown check
                    // Would need to track last execution time
                }
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

    fn get_action_tokens(&self, action: &Action) -> Vec<String> {
        match action {
            Action::Swap { token_in, token_out, .. } => vec![token_in.clone(), token_out.clone()],
            Action::Transfer { token, .. } => vec![token.clone()],
            Action::ApproveToken { token, .. } => vec![token.clone()],
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
        let roles = self.role_assignments
            .entry(principal)
            .or_insert_with(Vec::new);

        if !roles.contains(&role) {
            roles.push(role);
        }
    }

    pub fn revoke_role(&mut self, principal: &Principal, role: &Role) {
        if let Some(roles) = self.role_assignments.get_mut(principal) {
            roles.retain(|r| r != role);
        }
    }

    pub fn list_role_assignments(&self) -> Vec<(Principal, Role)> {
        let mut assignments = Vec::new();
        for (principal, roles) in &self.role_assignments {
            for role in roles {
                assignments.push((*principal, role.clone()));
            }
        }
        assignments
    }

    // CRUD operations for policies
    pub fn add_policy(&mut self, policy: Policy) -> u64 {
        let id = self.policies.len() as u64;
        self.policies.push(policy);
        id
    }

    pub fn update_policy(&mut self, index: usize, policy: Policy) -> bool {
        if index < self.policies.len() {
            self.policies[index] = policy;
            true
        } else {
            false
        }
    }

    pub fn remove_policy(&mut self, index: usize) -> bool {
        if index < self.policies.len() {
            self.policies.remove(index);
            true
        } else {
            false
        }
    }

    pub fn get_policies(&self) -> Vec<Policy> {
        self.policies.clone()
    }
}

impl Default for AccessControl {
    fn default() -> Self {
        Self::new()
    }
}
