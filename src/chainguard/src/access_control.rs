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

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    fn mock_principal(id: u8) -> Principal {
        let mut bytes = [0u8; 29];
        bytes[0] = id;
        Principal::from_slice(&bytes)
    }

    #[test]
    fn test_role_assignment() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);

        // Initially no roles
        assert!(!ac.has_role(&principal, &Role::Owner));

        // Assign role
        ac.assign_role(principal, Role::Owner);
        assert!(ac.has_role(&principal, &Role::Owner));

        // Revoke role
        ac.revoke_role(&principal, &Role::Owner);
        assert!(!ac.has_role(&principal, &Role::Owner));
    }

    #[test]
    fn test_multiple_roles() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);

        ac.assign_role(principal, Role::Owner);
        ac.assign_role(principal, Role::Operator);

        assert!(ac.has_role(&principal, &Role::Owner));
        assert!(ac.has_role(&principal, &Role::Operator));
        assert!(!ac.has_role(&principal, &Role::Viewer));
    }

    #[test]
    fn test_permissions_owner() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);
        ac.assign_role(principal, Role::Owner);

        // Owner has all permissions
        assert!(ac.has_permission(&principal, &Permission::Execute));
        assert!(ac.has_permission(&principal, &Permission::Configure));
        assert!(ac.has_permission(&principal, &Permission::ViewLogs));
        assert!(ac.has_permission(&principal, &Permission::Sign));
        assert!(ac.has_permission(&principal, &Permission::Emergency));
    }

    #[test]
    fn test_permissions_operator() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);
        ac.assign_role(principal, Role::Operator);

        assert!(ac.has_permission(&principal, &Permission::Execute));
        assert!(ac.has_permission(&principal, &Permission::Sign));
        assert!(ac.has_permission(&principal, &Permission::ViewLogs));
        assert!(!ac.has_permission(&principal, &Permission::Configure));
        assert!(!ac.has_permission(&principal, &Permission::Emergency));
    }

    #[test]
    fn test_permissions_viewer() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);
        ac.assign_role(principal, Role::Viewer);

        assert!(ac.has_permission(&principal, &Permission::ViewLogs));
        assert!(!ac.has_permission(&principal, &Permission::Execute));
        assert!(!ac.has_permission(&principal, &Permission::Configure));
        assert!(!ac.has_permission(&principal, &Permission::Sign));
        assert!(!ac.has_permission(&principal, &Permission::Emergency));
    }

    #[test]
    fn test_policy_add_remove() {
        let mut ac = AccessControl::new();

        let policy = Policy {
            name: "Test Policy".to_string(),
            conditions: vec![Condition::MaxAmount(1000)],
            action: PolicyAction::Allow,
            priority: 1,
        };

        let id = ac.add_policy(policy);
        assert_eq!(id, 0);
        assert_eq!(ac.get_policies().len(), 1);

        assert!(ac.remove_policy(0));
        assert_eq!(ac.get_policies().len(), 0);
    }

    #[test]
    fn test_policy_evaluation_allow() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);

        let policy = Policy {
            name: "Allow Small".to_string(),
            conditions: vec![Condition::MaxAmount(1000)],
            action: PolicyAction::Allow,
            priority: 1,
        };

        ac.add_policy(policy);

        let action = Action::Transfer {
            chain: "ethereum".to_string(),
            token: "USDC".to_string(),
            to: "0x123".to_string(),
            amount: 500,
        };

        let result = ac.evaluate_action(&action, &principal, 0);
        assert_eq!(result.decision, PolicyDecision::Allowed);
    }

    #[test]
    fn test_policy_evaluation_deny() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);

        let policy = Policy {
            name: "Deny Large".to_string(),
            conditions: vec![Condition::MaxAmount(1000)],
            action: PolicyAction::Deny,
            priority: 1,
        };

        ac.add_policy(policy);

        let action = Action::Transfer {
            chain: "ethereum".to_string(),
            token: "USDC".to_string(),
            to: "0x123".to_string(),
            amount: 2000,
        };

        let result = ac.evaluate_action(&action, &principal, 0);
        assert_eq!(result.decision, PolicyDecision::Denied);
    }

    #[test]
    fn test_policy_daily_limit() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);

        let policy = Policy {
            name: "Daily Limit".to_string(),
            conditions: vec![Condition::DailyLimit(5000)],
            action: PolicyAction::Allow,
            priority: 1,
        };

        ac.add_policy(policy);

        let action = Action::Transfer {
            chain: "ethereum".to_string(),
            token: "USDC".to_string(),
            to: "0x123".to_string(),
            amount: 1000,
        };

        // First transfer - within daily limit
        let result = ac.evaluate_action(&action, &principal, 3000);
        assert_eq!(result.decision, PolicyDecision::Allowed);

        // Second transfer - exceeds daily limit
        let result = ac.evaluate_action(&action, &principal, 4500);
        assert_eq!(result.decision, PolicyDecision::Denied);
    }

    #[test]
    fn test_policy_allowed_chains() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);

        let policy = Policy {
            name: "Allowed Chains".to_string(),
            conditions: vec![Condition::AllowedChains(vec!["ethereum".to_string(), "polygon".to_string()])],
            action: PolicyAction::Allow,
            priority: 1,
        };

        ac.add_policy(policy);

        // Allowed chain
        let action1 = Action::Transfer {
            chain: "ethereum".to_string(),
            token: "USDC".to_string(),
            to: "0x123".to_string(),
            amount: 1000,
        };
        let result1 = ac.evaluate_action(&action1, &principal, 0);
        assert_eq!(result1.decision, PolicyDecision::Allowed);

        // Disallowed chain
        let action2 = Action::Transfer {
            chain: "arbitrum".to_string(),
            token: "USDC".to_string(),
            to: "0x123".to_string(),
            amount: 1000,
        };
        let result2 = ac.evaluate_action(&action2, &principal, 0);
        assert_eq!(result2.decision, PolicyDecision::Denied);
    }

    #[test]
    fn test_policy_priority() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);

        // Lower priority (0) - should be evaluated first
        let deny_policy = Policy {
            name: "Deny Large".to_string(),
            conditions: vec![Condition::MaxAmount(10000)],
            action: PolicyAction::Deny,
            priority: 0,
        };

        // Higher priority (1) - should be evaluated second
        let allow_policy = Policy {
            name: "Allow Small".to_string(),
            conditions: vec![Condition::MaxAmount(1000)],
            action: PolicyAction::Allow,
            priority: 1,
        };

        ac.add_policy(allow_policy);
        ac.add_policy(deny_policy);

        let action = Action::Transfer {
            chain: "ethereum".to_string(),
            token: "USDC".to_string(),
            amount: 500,
            to: "0x123".to_string(),
        };

        let result = ac.evaluate_action(&action, &principal, 0);
        // Should match deny_policy first (lower priority number = higher priority)
        assert_eq!(result.decision, PolicyDecision::Denied);
        assert_eq!(result.matched_policy, Some("Deny Large".to_string()));
    }

    #[test]
    fn test_get_roles() {
        let mut ac = AccessControl::new();
        let principal = mock_principal(1);

        ac.assign_role(principal, Role::Owner);
        ac.assign_role(principal, Role::Operator);

        let roles = ac.get_roles(&principal);
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&Role::Owner));
        assert!(roles.contains(&Role::Operator));
    }

    #[test]
    fn test_list_role_assignments() {
        let mut ac = AccessControl::new();
        let principal1 = mock_principal(1);
        let principal2 = mock_principal(2);

        ac.assign_role(principal1, Role::Owner);
        ac.assign_role(principal2, Role::Operator);

        let assignments = ac.list_role_assignments();
        assert_eq!(assignments.len(), 2);
    }
}
