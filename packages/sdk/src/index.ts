/**
 * ChainGuard SDK
 *
 * TypeScript SDK for ChainGuard - Security middleware for AI agents on ICP
 *
 * @packageDocumentation
 */

export { ChainGuardClient, ChainGuardClientOptions } from './client';
export { idlFactory } from './idl';
export type {
  // Core types
  Role,
  Permission,
  Action,
  SwapAction,
  TransferAction,
  ApproveTokenAction,

  // Execution
  ActionResult,
  ExecutionResult,

  // Threshold signing
  PendingRequest,
  Signature,
  RequestStatus,

  // Policies
  Policy,
  PolicyAction,
  PolicyDecision,
  PolicyResult,
  Condition,

  // Audit
  AuditEntry,

  // Configuration
  ChainGuardConfig,

  // Results
  Result,

  // Service interface
  ChainGuardService,
} from './types';
