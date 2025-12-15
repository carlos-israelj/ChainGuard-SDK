use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use serde::Serialize;
use std::cell::RefCell;

use crate::types::*;

// Type aliases for stable memory
type Memory = VirtualMemory<DefaultMemoryImpl>;
type ConfigMemory = StableBTreeMap<u8, Vec<u8>, Memory>;
type RoleMemory = StableBTreeMap<Vec<u8>, Vec<u8>, Memory>;
type PolicyMemory = StableBTreeMap<u64, Vec<u8>, Memory>;
type PendingRequestMemory = StableBTreeMap<u64, Vec<u8>, Memory>;
type AuditMemory = StableBTreeMap<u64, Vec<u8>, Memory>;

const CONFIG_MEMORY_ID: MemoryId = MemoryId::new(0);
const ROLE_MEMORY_ID: MemoryId = MemoryId::new(1);
const POLICY_MEMORY_ID: MemoryId = MemoryId::new(2);
const PENDING_REQUEST_MEMORY_ID: MemoryId = MemoryId::new(3);
const AUDIT_MEMORY_ID: MemoryId = MemoryId::new(4);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static CONFIG_STORE: RefCell<ConfigMemory> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(CONFIG_MEMORY_ID)),
        )
    );

    static ROLE_STORE: RefCell<RoleMemory> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(ROLE_MEMORY_ID)),
        )
    );

    static POLICY_STORE: RefCell<PolicyMemory> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(POLICY_MEMORY_ID)),
        )
    );

    static PENDING_REQUEST_STORE: RefCell<PendingRequestMemory> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(PENDING_REQUEST_MEMORY_ID)),
        )
    );

    static AUDIT_STORE: RefCell<AuditMemory> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(AUDIT_MEMORY_ID)),
        )
    );
}

// Serializable state for upgrade persistence
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct StableState {
    pub config: Option<ChainGuardConfig>,
    pub role_assignments: Vec<(Principal, Vec<Role>)>,
    pub policies: Vec<Policy>,
    pub pending_requests: Vec<PendingRequest>,
    pub audit_entries: Vec<AuditEntry>,
    pub paused: bool,
    pub daily_volume: u64,
    pub last_reset: u64,
    pub executor_config: ExecutorConfig,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ExecutorConfig {
    pub key_name: String,
    pub derivation_path: Vec<Vec<u8>>,
}

// Store config
pub fn store_config(config: &ChainGuardConfig) -> Result<(), String> {
    let encoded = candid::encode_one(config)
        .map_err(|e| format!("Failed to encode config: {}", e))?;

    CONFIG_STORE.with(|store| {
        store.borrow_mut().insert(0, encoded);
    });

    Ok(())
}

// Load config
pub fn load_config() -> Option<ChainGuardConfig> {
    CONFIG_STORE.with(|store| {
        store.borrow().get(&0).and_then(|bytes| {
            candid::decode_one(&bytes).ok()
        })
    })
}

// Store role assignment
pub fn store_role(principal: &Principal, roles: &Vec<Role>) -> Result<(), String> {
    let key = principal.as_slice().to_vec();
    let encoded = candid::encode_one(roles)
        .map_err(|e| format!("Failed to encode roles: {}", e))?;

    ROLE_STORE.with(|store| {
        store.borrow_mut().insert(key, encoded);
    });

    Ok(())
}

// Load role assignment
pub fn load_role(principal: &Principal) -> Option<Vec<Role>> {
    let key = principal.as_slice().to_vec();
    ROLE_STORE.with(|store| {
        store.borrow().get(&key).and_then(|bytes| {
            candid::decode_one(&bytes).ok()
        })
    })
}

// Load all roles
pub fn load_all_roles() -> Vec<(Principal, Vec<Role>)> {
    ROLE_STORE.with(|store| {
        let store = store.borrow();
        store.iter().filter_map(|(key_bytes, value_bytes)| {
            let principal = Principal::try_from_slice(&key_bytes).ok()?;
            let roles: Vec<Role> = candid::decode_one(&value_bytes).ok()?;
            Some((principal, roles))
        }).collect()
    })
}

// Store policy
pub fn store_policy(index: u64, policy: &Policy) -> Result<(), String> {
    let encoded = candid::encode_one(policy)
        .map_err(|e| format!("Failed to encode policy: {}", e))?;

    POLICY_STORE.with(|store| {
        store.borrow_mut().insert(index, encoded);
    });

    Ok(())
}

// Load policy
pub fn load_policy(index: u64) -> Option<Policy> {
    POLICY_STORE.with(|store| {
        store.borrow().get(&index).and_then(|bytes| {
            candid::decode_one(&bytes).ok()
        })
    })
}

// Load all policies
pub fn load_all_policies() -> Vec<Policy> {
    POLICY_STORE.with(|store| {
        let store = store.borrow();
        store.iter().filter_map(|(_, value_bytes)| {
            candid::decode_one(&value_bytes).ok()
        }).collect()
    })
}

// Store pending request
pub fn store_pending_request(request: &PendingRequest) -> Result<(), String> {
    let encoded = candid::encode_one(request)
        .map_err(|e| format!("Failed to encode request: {}", e))?;

    PENDING_REQUEST_STORE.with(|store| {
        store.borrow_mut().insert(request.id, encoded);
    });

    Ok(())
}

// Load pending request
pub fn load_pending_request(id: u64) -> Option<PendingRequest> {
    PENDING_REQUEST_STORE.with(|store| {
        store.borrow().get(&id).and_then(|bytes| {
            candid::decode_one(&bytes).ok()
        })
    })
}

// Load all pending requests
pub fn load_all_pending_requests() -> Vec<PendingRequest> {
    PENDING_REQUEST_STORE.with(|store| {
        let store = store.borrow();
        store.iter().filter_map(|(_, value_bytes)| {
            candid::decode_one(&value_bytes).ok()
        }).collect()
    })
}

// Store audit entry
pub fn store_audit_entry(entry: &AuditEntry) -> Result<(), String> {
    let encoded = candid::encode_one(entry)
        .map_err(|e| format!("Failed to encode audit entry: {}", e))?;

    AUDIT_STORE.with(|store| {
        store.borrow_mut().insert(entry.id, encoded);
    });

    Ok(())
}

// Load audit entry
pub fn load_audit_entry(id: u64) -> Option<AuditEntry> {
    AUDIT_STORE.with(|store| {
        store.borrow().get(&id).and_then(|bytes| {
            candid::decode_one(&bytes).ok()
        })
    })
}

// Load all audit entries
pub fn load_all_audit_entries() -> Vec<AuditEntry> {
    AUDIT_STORE.with(|store| {
        let store = store.borrow();
        store.iter().filter_map(|(_, value_bytes)| {
            candid::decode_one(&value_bytes).ok()
        }).collect()
    })
}

// Clear all stable storage (for testing/reset)
pub fn clear_all_stable_storage() {
    CONFIG_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let keys: Vec<u8> = store.iter().map(|(k, _)| k).collect();
        for key in keys {
            store.remove(&key);
        }
    });

    ROLE_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let keys: Vec<Vec<u8>> = store.iter().map(|(k, _)| k).collect();
        for key in keys {
            store.remove(&key);
        }
    });

    POLICY_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let keys: Vec<u64> = store.iter().map(|(k, _)| k).collect();
        for key in keys {
            store.remove(&key);
        }
    });

    PENDING_REQUEST_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let keys: Vec<u64> = store.iter().map(|(k, _)| k).collect();
        for key in keys {
            store.remove(&key);
        }
    });

    AUDIT_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let keys: Vec<u64> = store.iter().map(|(k, _)| k).collect();
        for key in keys {
            store.remove(&key);
        }
    });
}
