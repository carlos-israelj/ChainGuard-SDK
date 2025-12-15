/**
 * TypeScript type definitions for ChainGuard SDK
 * Generated from chainguard.did
 */

import { Principal } from '@dfinity/principal';

export type Role = { Owner: null } | { Operator: null } | { Viewer: null };

export type Permission =
  | { Execute: null }
  | { Configure: null }
  | { ViewLogs: null }
  | { Sign: null }
  | { Emergency: null };

export interface SwapAction {
  chain: string;
  token_in: string;
  token_out: string;
  amount_in: bigint;
  min_amount_out: bigint;
  fee_tier: [] | [number];
}

export interface TransferAction {
  chain: string;
  token: string;
  to: string;
  amount: bigint;
}

export interface ApproveTokenAction {
  chain: string;
  token: string;
  spender: string;
  amount: bigint;
}

export interface BitcoinTransferAction {
  to: string;
  amount: bigint;
  network: string;
}

export type Action =
  | { Swap: SwapAction }
  | { Transfer: TransferAction }
  | { ApproveToken: ApproveTokenAction }
  | { BitcoinTransfer: BitcoinTransferAction };

export interface ExecutionResult {
  success: boolean;
  chain: string;
  tx_hash: [] | [string];
  error: [] | [string];
}

export type ActionResult =
  | { Executed: ExecutionResult }
  | { PendingSignatures: PendingRequest }
  | { Denied: { reason: string } };

export interface Signature {
  signer: Principal;
  signed_at: bigint;
}

export type RequestStatus =
  | { Pending: null }
  | { Approved: null }
  | { Executed: null }
  | { Expired: null }
  | { Rejected: null };

export interface PendingRequest {
  id: bigint;
  action: Action;
  requester: Principal;
  created_at: bigint;
  expires_at: bigint;
  required_signatures: number;
  collected_signatures: Signature[];
  status: RequestStatus;
}

export type PolicyDecision =
  | { Allowed: null }
  | { Denied: null }
  | { RequiresThreshold: null };

export interface PolicyResult {
  decision: PolicyDecision;
  matched_policy: [] | [string];
  reason: string;
}

export interface AuditEntry {
  id: bigint;
  timestamp: bigint;
  action_type: string;
  action_params: string;
  requester: Principal;
  policy_result: PolicyResult;
  threshold_request_id: [] | [bigint];
  execution_result: [] | [ExecutionResult];
}

export type Condition =
  | { MaxAmount: bigint }
  | { MinAmount: bigint }
  | { DailyLimit: bigint }
  | { AllowedTokens: string[] }
  | { AllowedChains: string[] }
  | { TimeWindow: { start: bigint; end: bigint } }
  | { Cooldown: bigint };

export type PolicyAction =
  | { Allow: null }
  | { Deny: null }
  | { RequireThreshold: { required: number; from_roles: Role[] } };

export interface Policy {
  name: string;
  conditions: Condition[];
  action: PolicyAction;
  priority: number;
}

export interface ChainGuardConfig {
  name: string;
  default_threshold: { required: number; total: number };
  supported_chains: string[];
  policies: Policy[];
}

export type Result<T = null> = { Ok: T } | { Err: string };

/**
 * ChainGuard Canister Interface
 */
export interface ChainGuardService {
  // Initialization
  initialize: (config: ChainGuardConfig) => Promise<Result>;

  // Role Management
  assign_role: (principal: Principal, role: Role) => Promise<Result>;
  revoke_role: (principal: Principal, role: Role) => Promise<Result>;
  get_roles: (principal: Principal) => Promise<Role[]>;
  list_role_assignments: () => Promise<Array<[Principal, Role]>>;

  // Policy Management
  add_policy: (policy: Policy) => Promise<Result<bigint>>;
  update_policy: (id: bigint, policy: Policy) => Promise<Result>;
  remove_policy: (id: bigint) => Promise<Result>;
  list_policies: () => Promise<Policy[]>;

  // Action Execution
  request_action: (action: Action) => Promise<ActionResult>;

  // Threshold Signing
  get_pending_requests: () => Promise<PendingRequest[]>;
  sign_request: (id: bigint) => Promise<Result<PendingRequest>>;
  reject_request: (id: bigint, reason: string) => Promise<Result>;

  // Audit
  get_audit_logs: (start: [] | [bigint], end: [] | [bigint]) => Promise<AuditEntry[]>;
  get_audit_entry: (id: bigint) => Promise<[] | [AuditEntry]>;

  // Emergency
  pause: () => Promise<Result>;
  resume: () => Promise<Result>;
  is_paused: () => Promise<boolean>;

  // Info
  get_config: () => Promise<[] | [ChainGuardConfig]>;
  get_eth_address: () => Promise<Result<string>>;
  get_bitcoin_address: (network: string) => Promise<Result<string>>;
}
