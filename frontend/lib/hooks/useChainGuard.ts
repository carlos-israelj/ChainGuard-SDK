'use client';

import { useState, useEffect, useCallback } from 'react';
import { Actor, HttpAgent } from '@dfinity/agent';
import { Ed25519KeyIdentity } from '@dfinity/identity';
import { Principal } from '@dfinity/principal';
import { idlFactory } from '../types/idl';
import type { ChainGuardService, ActionResult, PendingRequest, AuditEntry, Policy } from '../types/chainguard';

const CANISTER_ID = 'foxtk-ziaaa-aaaai-atthq-cai';
const HOST = 'https://icp-api.io';

export function useChainGuard() {
  const [actor, setActor] = useState<ChainGuardService | null>(null);
  const [principal, setPrincipal] = useState<Principal | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Initialize agent and actor
  useEffect(() => {
    async function init() {
      try {
        // Generate identity (in production, use stored identity)
        const identity = Ed25519KeyIdentity.generate();

        // Create agent
        const agent = await HttpAgent.create({
          host: HOST,
          identity,
        });

        // Create actor
        const actorInstance = Actor.createActor<ChainGuardService>(idlFactory as any, {
          agent,
          canisterId: Principal.fromText(CANISTER_ID),
        });

        setActor(actorInstance);
        setPrincipal(identity.getPrincipal());
        setLoading(false);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to initialize');
        setLoading(false);
      }
    }

    init();
  }, []);

  // Actions
  const requestAction = useCallback(async (action: any): Promise<ActionResult | null> => {
    if (!actor) return null;
    try {
      return await actor.request_action(action);
    } catch (err) {
      console.error('Request action failed:', err);
      return null;
    }
  }, [actor]);

  const getPendingRequests = useCallback(async (): Promise<PendingRequest[]> => {
    if (!actor) return [];
    try {
      return await actor.get_pending_requests();
    } catch (err) {
      console.error('Get pending requests failed:', err);
      return [];
    }
  }, [actor]);

  const signRequest = useCallback(async (requestId: bigint) => {
    if (!actor) return null;
    try {
      return await actor.sign_request(requestId);
    } catch (err) {
      console.error('Sign request failed:', err);
      return null;
    }
  }, [actor]);

  const getAuditLogs = useCallback(async (start?: bigint, end?: bigint): Promise<AuditEntry[]> => {
    if (!actor) return [];
    try {
      return await actor.get_audit_logs(start ? [start] : [], end ? [end] : []);
    } catch (err) {
      console.error('Get audit logs failed:', err);
      return [];
    }
  }, [actor]);

  const listPolicies = useCallback(async (): Promise<Policy[]> => {
    if (!actor) return [];
    try {
      return await actor.list_policies();
    } catch (err) {
      console.error('List policies failed:', err);
      return [];
    }
  }, [actor]);

  const getConfig = useCallback(async () => {
    if (!actor) return null;
    try {
      const result = await actor.get_config();
      return result.length > 0 ? result[0] : null;
    } catch (err) {
      console.error('Get config failed:', err);
      return null;
    }
  }, [actor]);

  const isPaused = useCallback(async (): Promise<boolean> => {
    if (!actor) return false;
    try {
      return await actor.is_paused();
    } catch (err) {
      console.error('Is paused check failed:', err);
      return false;
    }
  }, [actor]);

  return {
    actor,
    principal,
    loading,
    error,
    // Methods
    requestAction,
    getPendingRequests,
    signRequest,
    getAuditLogs,
    listPolicies,
    getConfig,
    isPaused,
  };
}
