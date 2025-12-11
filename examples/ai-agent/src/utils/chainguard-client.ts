/**
 * ChainGuard Client - Wrapper for interacting with ChainGuard canister
 */

import { Actor, HttpAgent, Identity } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { idlFactory } from '../types/idl';
import {
  ChainGuardService,
  Action,
  ActionResult,
  Policy,
  PendingRequest,
  AuditEntry,
  Role,
} from '../types/chainguard';

export interface ChainGuardClientConfig {
  canisterId: string;
  identity: Identity;
  host?: string;
}

export class ChainGuardClient {
  private actor: ChainGuardService;
  private agent: HttpAgent;
  public readonly canisterId: Principal;

  constructor(config: ChainGuardClientConfig) {
    this.canisterId = Principal.fromText(config.canisterId);

    // Create HTTP agent
    this.agent = HttpAgent.createSync({
      host: config.host || 'https://icp-api.io',
      identity: config.identity,
    });

    // For local development, fetch root key
    if (config.host?.includes('localhost') || config.host?.includes('127.0.0.1')) {
      this.agent.fetchRootKey().catch(err => {
        console.warn('Unable to fetch root key. Check if the local replica is running.');
        console.error(err);
      });
    }

    // Create actor
    this.actor = Actor.createActor<ChainGuardService>(idlFactory as any, {
      agent: this.agent,
      canisterId: this.canisterId,
    });
  }

  /**
   * Get the principal ID of the current identity
   */
  getPrincipal(): Principal {
    const principal = this.agent.getPrincipal();
    // If it's a promise, await it; otherwise return directly
    return principal as any as Principal;
  }

  // ==================== Action Execution ====================

  /**
   * Request an action (Transfer, Swap, or ApproveToken)
   */
  async requestAction(action: Action): Promise<ActionResult> {
    console.log(`[ChainGuard] Requesting action:`, this.formatAction(action));
    const result = await this.actor.request_action(action);
    console.log(`[ChainGuard] Action result:`, this.formatActionResult(result));
    return result;
  }

  /**
   * Execute a token transfer
   */
  async transfer(
    chain: string,
    token: string,
    to: string,
    amount: bigint
  ): Promise<ActionResult> {
    return this.requestAction({
      Transfer: { chain, token, to, amount },
    });
  }

  /**
   * Execute a token swap
   */
  async swap(
    chain: string,
    tokenIn: string,
    tokenOut: string,
    amountIn: bigint,
    minAmountOut: bigint,
    feeTier?: number
  ): Promise<ActionResult> {
    return this.requestAction({
      Swap: {
        chain,
        token_in: tokenIn,
        token_out: tokenOut,
        amount_in: amountIn,
        min_amount_out: minAmountOut,
        fee_tier: feeTier ? [feeTier] : [],
      },
    });
  }

  // ==================== Threshold Signing ====================

  /**
   * Get all pending signature requests
   */
  async getPendingRequests(): Promise<PendingRequest[]> {
    return this.actor.get_pending_requests();
  }

  /**
   * Sign a pending request
   */
  async signRequest(requestId: bigint): Promise<PendingRequest> {
    console.log(`[ChainGuard] Signing request ${requestId}...`);
    const result = await this.actor.sign_request(requestId);

    if ('Err' in result) {
      throw new Error(`Failed to sign request: ${result.Err}`);
    }

    console.log(`[ChainGuard] Request signed successfully`);
    return result.Ok;
  }

  /**
   * Reject a pending request
   */
  async rejectRequest(requestId: bigint, reason: string): Promise<void> {
    console.log(`[ChainGuard] Rejecting request ${requestId}: ${reason}`);
    const result = await this.actor.reject_request(requestId, reason);

    if ('Err' in result) {
      throw new Error(`Failed to reject request: ${result.Err}`);
    }
  }

  // ==================== Policy Management ====================

  /**
   * List all policies
   */
  async listPolicies(): Promise<Policy[]> {
    return this.actor.list_policies();
  }

  /**
   * Add a new policy
   */
  async addPolicy(policy: Policy): Promise<bigint> {
    console.log(`[ChainGuard] Adding policy: ${policy.name}`);
    const result = await this.actor.add_policy(policy);

    if ('Err' in result) {
      throw new Error(`Failed to add policy: ${result.Err}`);
    }

    console.log(`[ChainGuard] Policy added with ID: ${result.Ok}`);
    return result.Ok;
  }

  // ==================== Role Management ====================

  /**
   * Get roles for a principal
   */
  async getRoles(principal: Principal): Promise<Role[]> {
    return this.actor.get_roles(principal);
  }

  /**
   * List all role assignments
   */
  async listRoleAssignments(): Promise<Array<[Principal, Role]>> {
    return this.actor.list_role_assignments();
  }

  // ==================== Audit ====================

  /**
   * Get audit logs within a time range
   */
  async getAuditLogs(start?: bigint, end?: bigint): Promise<AuditEntry[]> {
    return this.actor.get_audit_logs(start ? [start] : [], end ? [end] : []);
  }

  /**
   * Get a specific audit entry
   */
  async getAuditEntry(id: bigint): Promise<AuditEntry | null> {
    const result = await this.actor.get_audit_entry(id);
    return result.length > 0 ? result[0] : null;
  }

  // ==================== System Info ====================

  /**
   * Check if the system is paused
   */
  async isPaused(): Promise<boolean> {
    return this.actor.is_paused();
  }

  /**
   * Get canister configuration
   */
  async getConfig() {
    const result = await this.actor.get_config();
    return result.length > 0 ? result[0] : null;
  }

  // ==================== Utility Methods ====================

  private formatAction(action: Action): string {
    if ('Transfer' in action) {
      const t = action.Transfer;
      return `Transfer ${t.amount} ${t.token} on ${t.chain} to ${t.to}`;
    } else if ('Swap' in action) {
      const s = action.Swap;
      return `Swap ${s.amount_in} ${s.token_in} → ${s.token_out} on ${s.chain}`;
    } else if ('ApproveToken' in action) {
      const a = action.ApproveToken;
      return `Approve ${a.amount} ${a.token} for ${a.spender} on ${a.chain}`;
    }
    return 'Unknown action';
  }

  private formatActionResult(result: ActionResult): string {
    if ('Executed' in result) {
      const exec = result.Executed;
      if (exec.success && exec.tx_hash.length > 0) {
        return `✅ Executed: ${exec.tx_hash[0]}`;
      } else if (exec.error && exec.error.length > 0) {
        return `❌ Failed: ${exec.error[0]}`;
      }
      return `Status: ${exec.success ? 'Success' : 'Failed'}`;
    } else if ('PendingSignatures' in result) {
      const pending = result.PendingSignatures;
      return `⏳ Pending signatures: ${pending.collected_signatures.length}/${pending.required_signatures}`;
    } else if ('Denied' in result) {
      return `🚫 Denied: ${result.Denied.reason}`;
    }
    return 'Unknown result';
  }
}
