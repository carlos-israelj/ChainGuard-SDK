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

impl Default for ThresholdSigner {
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

    #[test]
    fn test_create_request() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let action = mock_action();

        let request = ts.create_request(action.clone(), requester, 2, 1000);

        assert_eq!(request.id, 0);
        assert_eq!(request.requester, requester);
        assert_eq!(request.required_signatures, 2);
        assert_eq!(request.collected_signatures.len(), 0);
        assert_eq!(request.status, RequestStatus::Pending);
        assert_eq!(request.created_at, 1000);
        assert_eq!(request.expires_at, 1000 + 86400); // default expiry
    }

    #[test]
    fn test_sign_request_success() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let signer1 = mock_principal(2);
        let action = mock_action();

        let request = ts.create_request(action, requester, 2, 1000);
        let request_id = request.id;

        // First signature
        let result = ts.sign_request(request_id, signer1, 1500);
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.collected_signatures.len(), 1);
        assert_eq!(updated.status, RequestStatus::Pending);
    }

    #[test]
    fn test_sign_request_threshold_reached() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let signer1 = mock_principal(2);
        let signer2 = mock_principal(3);
        let action = mock_action();

        let request = ts.create_request(action, requester, 2, 1000);
        let request_id = request.id;

        // First signature
        ts.sign_request(request_id, signer1, 1500).unwrap();

        // Second signature - threshold reached
        let result = ts.sign_request(request_id, signer2, 1600);
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.collected_signatures.len(), 2);
        assert_eq!(updated.status, RequestStatus::Approved);
    }

    #[test]
    fn test_sign_request_duplicate_signer() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let signer = mock_principal(2);
        let action = mock_action();

        let request = ts.create_request(action, requester, 2, 1000);
        let request_id = request.id;

        // First signature
        ts.sign_request(request_id, signer, 1500).unwrap();

        // Try to sign again with same signer
        let result = ts.sign_request(request_id, signer, 1600);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Already signed by this principal");
    }

    #[test]
    fn test_sign_request_expired() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let signer = mock_principal(2);
        let action = mock_action();

        let request = ts.create_request(action, requester, 2, 1000);
        let request_id = request.id;

        // Try to sign after expiry
        let result = ts.sign_request(request_id, signer, 1000 + 86400 + 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Request expired");

        // Check status was updated
        let req = ts.get_request(request_id).unwrap();
        assert_eq!(req.status, RequestStatus::Expired);
    }

    #[test]
    fn test_sign_request_not_found() {
        let mut ts = ThresholdSigner::new();
        let signer = mock_principal(1);

        let result = ts.sign_request(999, signer, 1000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Request not found");
    }

    #[test]
    fn test_reject_request() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let action = mock_action();

        let request = ts.create_request(action, requester, 2, 1000);
        let request_id = request.id;

        let result = ts.reject_request(request_id, "Security concern".to_string());
        assert!(result.is_ok());

        let req = ts.get_request(request_id).unwrap();
        assert_eq!(req.status, RequestStatus::Rejected);
    }

    #[test]
    fn test_mark_executed() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let action = mock_action();

        let request = ts.create_request(action, requester, 2, 1000);
        let request_id = request.id;

        let result = ts.mark_executed(request_id);
        assert!(result.is_ok());

        let req = ts.get_request(request_id).unwrap();
        assert_eq!(req.status, RequestStatus::Executed);
    }

    #[test]
    fn test_get_pending_requests() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let action = mock_action();

        // Create multiple requests
        ts.create_request(action.clone(), requester, 2, 1000);
        ts.create_request(action.clone(), requester, 2, 2000);
        ts.create_request(action.clone(), requester, 2, 3000);

        // Mark one as executed
        ts.mark_executed(1).unwrap();

        let pending = ts.get_pending_requests();
        assert_eq!(pending.len(), 2);
        assert!(pending.iter().all(|r| r.status == RequestStatus::Pending));
    }

    #[test]
    fn test_is_approved() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let signer1 = mock_principal(2);
        let signer2 = mock_principal(3);
        let action = mock_action();

        let request = ts.create_request(action, requester, 2, 1000);
        let request_id = request.id;

        assert!(!ts.is_approved(request_id));

        // Add signatures
        ts.sign_request(request_id, signer1, 1500).unwrap();
        assert!(!ts.is_approved(request_id));

        ts.sign_request(request_id, signer2, 1600).unwrap();
        assert!(ts.is_approved(request_id));
    }

    #[test]
    fn test_cleanup_expired() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let action = mock_action();

        // Create requests at different times
        ts.create_request(action.clone(), requester, 2, 1000);
        ts.create_request(action.clone(), requester, 2, 2000);
        ts.create_request(action.clone(), requester, 2, 3000);

        // Cleanup at time that expires first two
        ts.cleanup_expired(1000 + 86400 + 1);

        let req0 = ts.get_request(0).unwrap();
        let req1 = ts.get_request(1).unwrap();
        let req2 = ts.get_request(2).unwrap();

        assert_eq!(req0.status, RequestStatus::Expired);
        assert_eq!(req1.status, RequestStatus::Pending);
        assert_eq!(req2.status, RequestStatus::Pending);
    }

    #[test]
    fn test_multiple_requests_different_ids() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let action = mock_action();

        let req1 = ts.create_request(action.clone(), requester, 2, 1000);
        let req2 = ts.create_request(action.clone(), requester, 2, 2000);
        let req3 = ts.create_request(action, requester, 2, 3000);

        assert_eq!(req1.id, 0);
        assert_eq!(req2.id, 1);
        assert_eq!(req3.id, 2);
    }

    #[test]
    fn test_sign_after_approval() {
        let mut ts = ThresholdSigner::new();
        let requester = mock_principal(1);
        let signer1 = mock_principal(2);
        let signer2 = mock_principal(3);
        let signer3 = mock_principal(4);
        let action = mock_action();

        let request = ts.create_request(action, requester, 2, 1000);
        let request_id = request.id;

        // Reach threshold
        ts.sign_request(request_id, signer1, 1500).unwrap();
        ts.sign_request(request_id, signer2, 1600).unwrap();

        // Try to sign after approval
        let result = ts.sign_request(request_id, signer3, 1700);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not pending"));
    }
}
