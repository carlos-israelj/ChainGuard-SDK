/**
 * ChainGuard SDK Client
 *
 * Main client for interacting with ChainGuard canister on Internet Computer Protocol
 */

import { Actor, HttpAgent, Identity } from '@dfinity/agent';
import { Ed25519KeyIdentity } from '@dfinity/identity';
import { Principal } from '@dfinity/principal';
import { idlFactory } from './idl';
import type {
  ChainGuardService,
  Action,
  ActionResult,
  PendingRequest,
  AuditEntry,
  Policy,
  ChainGuardConfig,
  Role,
  Result,
} from './types';

export interface ChainGuardClientOptions {
  canisterId: string;
  host?: string;
  identity?: Identity;
}

export class ChainGuardClient {
  private actor: ChainGuardService;
  private principal: Principal;
  private canisterId: string;

  constructor(options: ChainGuardClientOptions) {
    this.canisterId = options.canisterId;
    const host = options.host || 'https://icp-api.io';
    const identity = options.identity || Ed25519KeyIdentity.generate();

    // Create HTTP agent
    const agent = HttpAgent.createSync({
      host,
      identity,
    });

    // Create actor
    this.actor = Actor.createActor<ChainGuardService>(idlFactory as any, {
      agent,
      canisterId: Principal.fromText(this.canisterId),
    });

    this.principal = identity.getPrincipal();
  }

  /**
   * Get the principal of the current identity
   */
  getPrincipal(): Principal {
    return this.principal;
  }

  /**
   * Get the canister ID
   */
  getCanisterId(): string {
    return this.canisterId;
  }

  // ============ Initialization ============

  /**
   * Initialize the ChainGuard canister with configuration
   */
  async initialize(config: ChainGuardConfig): Promise<Result> {
    return await this.actor.initialize(config);
  }

  // ============ Role Management ============

  /**
   * Assign a role to a principal
   */
  async assignRole(principal: Principal, role: Role): Promise<Result> {
    return await this.actor.assign_role(principal, role);
  }

  /**
   * Revoke a role from a principal
   */
  async revokeRole(principal: Principal, role: Role): Promise<Result> {
    return await this.actor.revoke_role(principal, role);
  }

  /**
   * Get roles for a principal
   */
  async getRoles(principal: Principal): Promise<Role[]> {
    return await this.actor.get_roles(principal);
  }

  /**
   * List all role assignments
   */
  async listRoleAssignments(): Promise<Array<[Principal, Role]>> {
    return await this.actor.list_role_assignments();
  }

  // ============ Policy Management ============

  /**
   * Add a new policy
   */
  async addPolicy(policy: Policy): Promise<Result<bigint>> {
    return await this.actor.add_policy(policy);
  }

  /**
   * Update an existing policy
   */
  async updatePolicy(id: bigint, policy: Policy): Promise<Result> {
    return await this.actor.update_policy(id, policy);
  }

  /**
   * Remove a policy
   */
  async removePolicy(id: bigint): Promise<Result> {
    return await this.actor.remove_policy(id);
  }

  /**
   * List all policies
   */
  async listPolicies(): Promise<Policy[]> {
    return await this.actor.list_policies();
  }

  // ============ Action Execution ============

  /**
   * Request an action (transfer, swap, approve token)
   * Returns ActionResult which can be:
   * - Executed: Action was executed immediately
   * - PendingSignatures: Action requires threshold signatures
   * - Denied: Action was denied by policy
   */
  async requestAction(action: Action): Promise<ActionResult> {
    return await this.actor.request_action(action);
  }

  /**
   * Helper: Request a transfer action
   */
  async transfer(
    chain: string,
    token: string,
    to: string,
    amount: bigint
  ): Promise<ActionResult> {
    return await this.requestAction({
      Transfer: { chain, token, to, amount },
    });
  }

  /**
   * Helper: Request a swap action
   */
  async swap(
    chain: string,
    tokenIn: string,
    tokenOut: string,
    amountIn: bigint,
    minAmountOut: bigint,
    feeTier?: number
  ): Promise<ActionResult> {
    return await this.requestAction({
      Swap: {
        chain,
        token_in: tokenIn,
        token_out: tokenOut,
        amount_in: amountIn,
        min_amount_out: minAmountOut,
        fee_tier: feeTier !== undefined ? [feeTier] : [],
      },
    });
  }

  /**
   * Helper: Request a token approval action
   */
  async approveToken(
    chain: string,
    token: string,
    spender: string,
    amount: bigint
  ): Promise<ActionResult> {
    return await this.requestAction({
      ApproveToken: { chain, token, spender, amount },
    });
  }

  // ============ Threshold Signing ============

  /**
   * Get all pending threshold signature requests
   */
  async getPendingRequests(): Promise<PendingRequest[]> {
    return await this.actor.get_pending_requests();
  }

  /**
   * Sign a pending request
   */
  async signRequest(requestId: bigint): Promise<Result<PendingRequest>> {
    return await this.actor.sign_request(requestId);
  }

  /**
   * Reject a pending request
   */
  async rejectRequest(requestId: bigint, reason: string): Promise<Result> {
    return await this.actor.reject_request(requestId, reason);
  }

  // ============ Audit & Monitoring ============

  /**
   * Get audit logs within a time range
   * @param start - Optional start timestamp (nanoseconds)
   * @param end - Optional end timestamp (nanoseconds)
   */
  async getAuditLogs(start?: bigint, end?: bigint): Promise<AuditEntry[]> {
    return await this.actor.get_audit_logs(
      start !== undefined ? [start] : [],
      end !== undefined ? [end] : []
    );
  }

  /**
   * Get a specific audit entry by ID
   */
  async getAuditEntry(id: bigint): Promise<AuditEntry | null> {
    const result = await this.actor.get_audit_entry(id);
    return result.length > 0 ? result[0] ?? null : null;
  }

  /**
   * Get canister configuration
   */
  async getConfig(): Promise<ChainGuardConfig | null> {
    const result = await this.actor.get_config();
    return result.length > 0 ? result[0] ?? null : null;
  }

  /**
   * Check if the system is paused
   */
  async isPaused(): Promise<boolean> {
    return await this.actor.is_paused();
  }

  // ============ Emergency Controls ============

  /**
   * Pause all operations (requires Owner role)
   */
  async pause(): Promise<Result> {
    return await this.actor.pause();
  }

  /**
   * Resume operations (requires Owner role)
   */
  async resume(): Promise<Result> {
    return await this.actor.resume();
  }
}
