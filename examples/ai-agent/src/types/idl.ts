/**
 * Candid IDL factory for ChainGuard canister
 */

import { IDL } from '@dfinity/candid';

export const idlFactory = ({ IDL }: { IDL: typeof import('@dfinity/candid').IDL }) => {
  const Role = IDL.Variant({
    Owner: IDL.Null,
    Operator: IDL.Null,
    Viewer: IDL.Null,
  });

  const Permission = IDL.Variant({
    Execute: IDL.Null,
    Configure: IDL.Null,
    ViewLogs: IDL.Null,
    Sign: IDL.Null,
    Emergency: IDL.Null,
  });

  const Action = IDL.Variant({
    Swap: IDL.Record({
      chain: IDL.Text,
      token_in: IDL.Text,
      token_out: IDL.Text,
      amount_in: IDL.Nat64,
      min_amount_out: IDL.Nat64,
      fee_tier: IDL.Opt(IDL.Nat32),
    }),
    Transfer: IDL.Record({
      chain: IDL.Text,
      token: IDL.Text,
      to: IDL.Text,
      amount: IDL.Nat64,
    }),
    ApproveToken: IDL.Record({
      chain: IDL.Text,
      token: IDL.Text,
      spender: IDL.Text,
      amount: IDL.Nat64,
    }),
  });

  const ExecutionResult = IDL.Record({
    success: IDL.Bool,
    chain: IDL.Text,
    tx_hash: IDL.Opt(IDL.Text),
    error: IDL.Opt(IDL.Text),
  });

  const Signature = IDL.Record({
    signer: IDL.Principal,
    signed_at: IDL.Nat64,
  });

  const RequestStatus = IDL.Variant({
    Pending: IDL.Null,
    Approved: IDL.Null,
    Executed: IDL.Null,
    Expired: IDL.Null,
    Rejected: IDL.Null,
  });

  const PendingRequest = IDL.Record({
    id: IDL.Nat64,
    action: Action,
    requester: IDL.Principal,
    created_at: IDL.Nat64,
    expires_at: IDL.Nat64,
    required_signatures: IDL.Nat8,
    collected_signatures: IDL.Vec(Signature),
    status: RequestStatus,
  });

  const ActionResult = IDL.Variant({
    Executed: ExecutionResult,
    PendingSignatures: PendingRequest,
    Denied: IDL.Record({ reason: IDL.Text }),
  });

  const PolicyDecision = IDL.Variant({
    Allowed: IDL.Null,
    Denied: IDL.Null,
    RequiresThreshold: IDL.Null,
  });

  const PolicyResult = IDL.Record({
    decision: PolicyDecision,
    matched_policy: IDL.Opt(IDL.Text),
    reason: IDL.Text,
  });

  const AuditEntry = IDL.Record({
    id: IDL.Nat64,
    timestamp: IDL.Nat64,
    action_type: IDL.Text,
    action_params: IDL.Text,
    requester: IDL.Principal,
    policy_result: PolicyResult,
    threshold_request_id: IDL.Opt(IDL.Nat64),
    execution_result: IDL.Opt(ExecutionResult),
  });

  const Condition = IDL.Variant({
    MaxAmount: IDL.Nat64,
    MinAmount: IDL.Nat64,
    DailyLimit: IDL.Nat64,
    AllowedTokens: IDL.Vec(IDL.Text),
    AllowedChains: IDL.Vec(IDL.Text),
    TimeWindow: IDL.Record({ start: IDL.Nat64, end: IDL.Nat64 }),
    Cooldown: IDL.Nat64,
  });

  const PolicyAction = IDL.Variant({
    Allow: IDL.Null,
    Deny: IDL.Null,
    RequireThreshold: IDL.Record({
      required: IDL.Nat8,
      from_roles: IDL.Vec(Role),
    }),
  });

  const Policy = IDL.Record({
    name: IDL.Text,
    conditions: IDL.Vec(Condition),
    action: PolicyAction,
    priority: IDL.Nat32,
  });

  const ChainGuardConfig = IDL.Record({
    name: IDL.Text,
    default_threshold: IDL.Record({
      required: IDL.Nat8,
      total: IDL.Nat8,
    }),
    supported_chains: IDL.Vec(IDL.Text),
    policies: IDL.Vec(Policy),
  });

  const Result = IDL.Variant({
    Ok: IDL.Null,
    Err: IDL.Text,
  });

  const ResultWithId = IDL.Variant({
    Ok: IDL.Nat64,
    Err: IDL.Text,
  });

  const ResultWithRequest = IDL.Variant({
    Ok: PendingRequest,
    Err: IDL.Text,
  });

  return IDL.Service({
    // Initialization
    initialize: IDL.Func([ChainGuardConfig], [Result], []),

    // Role Management
    assign_role: IDL.Func([IDL.Principal, Role], [Result], []),
    revoke_role: IDL.Func([IDL.Principal, Role], [Result], []),
    get_roles: IDL.Func([IDL.Principal], [IDL.Vec(Role)], ['query']),
    list_role_assignments: IDL.Func([], [IDL.Vec(IDL.Tuple(IDL.Principal, Role))], ['query']),

    // Policy Management
    add_policy: IDL.Func([Policy], [ResultWithId], []),
    update_policy: IDL.Func([IDL.Nat64, Policy], [Result], []),
    remove_policy: IDL.Func([IDL.Nat64], [Result], []),
    list_policies: IDL.Func([], [IDL.Vec(Policy)], ['query']),

    // Action Execution
    request_action: IDL.Func([Action], [ActionResult], []),

    // Threshold Signing
    get_pending_requests: IDL.Func([], [IDL.Vec(PendingRequest)], ['query']),
    sign_request: IDL.Func([IDL.Nat64], [ResultWithRequest], []),
    reject_request: IDL.Func([IDL.Nat64, IDL.Text], [Result], []),

    // Audit
    get_audit_logs: IDL.Func([IDL.Opt(IDL.Nat64), IDL.Opt(IDL.Nat64)], [IDL.Vec(AuditEntry)], ['query']),
    get_audit_entry: IDL.Func([IDL.Nat64], [IDL.Opt(AuditEntry)], ['query']),

    // Emergency
    pause: IDL.Func([], [Result], []),
    resume: IDL.Func([], [Result], []),
    is_paused: IDL.Func([], [IDL.Bool], ['query']),

    // Info
    get_config: IDL.Func([], [IDL.Opt(ChainGuardConfig)], ['query']),
  });
};
