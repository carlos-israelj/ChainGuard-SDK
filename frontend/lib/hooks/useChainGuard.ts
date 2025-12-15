'use client';

import { useState, useEffect, useCallback } from 'react';
import { ChainGuardClient } from '@chainguarsdk/sdk';
import type {
  ActionResult,
  PendingRequest,
  AuditEntry,
  Policy,
  Action,
  ChainGuardConfig
} from '@chainguarsdk/sdk';
import { Ed25519KeyIdentity } from '@dfinity/identity';
import { Principal } from '@dfinity/principal';

const CANISTER_ID = 'foxtk-ziaaa-aaaai-atthq-cai';
const HOST = 'https://icp-api.io';

export function useChainGuard() {
  const [client, setClient] = useState<ChainGuardClient | null>(null);
  const [principal, setPrincipal] = useState<Principal | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Initialize ChainGuard client
  useEffect(() => {
    async function init() {
      try {
        // Generate identity (in production, use Internet Identity or stored identity)
        const identity = Ed25519KeyIdentity.generate();

        // Create ChainGuard client with SDK
        const clientInstance = new ChainGuardClient({
          canisterId: CANISTER_ID,
          host: HOST,
          identity,
        });

        setClient(clientInstance);
        setPrincipal(identity.getPrincipal());
        setLoading(false);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to initialize ChainGuard client');
        setLoading(false);
      }
    }

    init();
  }, []);

  // Request action (transfer, swap, approve)
  const requestAction = useCallback(async (action: Action): Promise<ActionResult | null> => {
    if (!client) return null;
    try {
      return await client.requestAction(action);
    } catch (err) {
      console.error('Request action failed:', err);
      return null;
    }
  }, [client]);

  // Get pending threshold signature requests
  const getPendingRequests = useCallback(async (): Promise<PendingRequest[]> => {
    if (!client) return [];
    try {
      return await client.getPendingRequests();
    } catch (err) {
      console.error('Get pending requests failed:', err);
      return [];
    }
  }, [client]);

  // Sign a pending request
  const signRequest = useCallback(async (requestId: bigint) => {
    if (!client) return null;
    try {
      return await client.signRequest(requestId);
    } catch (err) {
      console.error('Sign request failed:', err);
      return null;
    }
  }, [client]);

  // Get audit logs
  const getAuditLogs = useCallback(async (start?: bigint, end?: bigint): Promise<AuditEntry[]> => {
    if (!client) return [];
    try {
      return await client.getAuditLogs(start, end);
    } catch (err) {
      console.error('Get audit logs failed:', err);
      return [];
    }
  }, [client]);

  // List all policies
  const listPolicies = useCallback(async (): Promise<Policy[]> => {
    if (!client) return [];
    try {
      return await client.listPolicies();
    } catch (err) {
      console.error('List policies failed:', err);
      return [];
    }
  }, [client]);

  // Get canister configuration
  const getConfig = useCallback(async (): Promise<ChainGuardConfig | null> => {
    if (!client) return null;
    try {
      const result = await client.getConfig();
      return result || null;
    } catch (err) {
      console.error('Get config failed:', err);
      return null;
    }
  }, [client]);

  // Check if canister is paused
  const isPaused = useCallback(async (): Promise<boolean> => {
    if (!client) return false;
    try {
      return await client.isPaused();
    } catch (err) {
      console.error('Check paused status failed:', err);
      return false;
    }
  }, [client]);

  // Helper methods for common actions
  const transfer = useCallback(async (
    chain: string,
    token: string,
    to: string,
    amount: bigint
  ): Promise<ActionResult | null> => {
    if (!client) return null;
    try {
      return await client.transfer(chain, token, to, amount);
    } catch (err) {
      console.error('Transfer failed:', err);
      return null;
    }
  }, [client]);

  const swap = useCallback(async (
    chain: string,
    tokenIn: string,
    tokenOut: string,
    amountIn: bigint,
    minAmountOut: bigint,
    feeTier?: number
  ): Promise<ActionResult | null> => {
    if (!client) return null;
    try {
      return await client.swap(chain, tokenIn, tokenOut, amountIn, minAmountOut, feeTier);
    } catch (err) {
      console.error('Swap failed:', err);
      return null;
    }
  }, [client]);

  const approveToken = useCallback(async (
    chain: string,
    token: string,
    spender: string,
    amount: bigint
  ): Promise<ActionResult | null> => {
    if (!client) return null;
    try {
      return await client.approveToken(chain, token, spender, amount);
    } catch (err) {
      console.error('Approve token failed:', err);
      return null;
    }
  }, [client]);

  return {
    client,
    principal,
    loading,
    error,
    // Actions
    requestAction,
    transfer,
    swap,
    approveToken,
    // Threshold signatures
    getPendingRequests,
    signRequest,
    // Audit & monitoring
    getAuditLogs,
    listPolicies,
    getConfig,
    isPaused,
  };
}
