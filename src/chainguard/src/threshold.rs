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
