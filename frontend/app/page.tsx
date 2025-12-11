'use client';

import { useEffect, useState } from 'react';
import { useChainGuard } from '@/lib/hooks/useChainGuard';
import Link from 'next/link';

export default function Dashboard() {
  const {
    loading,
    error,
    principal,
    getConfig,
    isPaused,
    getPendingRequests,
    getAuditLogs,
    listPolicies
  } = useChainGuard();

  const [systemStatus, setSystemStatus] = useState<any>(null);
  const [stats, setStats] = useState({
    pendingRequests: 0,
    totalActions: 0,
    activePolicies: 0,
    paused: false,
  });

  useEffect(() => {
    async function loadData() {
      const [config, paused, pending, logs, policies] = await Promise.all([
        getConfig(),
        isPaused(),
        getPendingRequests(),
        getAuditLogs(),
        listPolicies(),
      ]);

      setSystemStatus(config);
      setStats({
        pendingRequests: pending.length,
        totalActions: logs.length,
        activePolicies: policies.length,
        paused,
      });
    }

    if (!loading) {
      loadData();
    }
  }, [loading, getConfig, isPaused, getPendingRequests, getAuditLogs, listPolicies]);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-4 border-blue-600 mx-auto"></div>
          <p className="mt-4 text-gray-700 font-medium">Connecting to ChainGuard...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gradient-to-br from-red-50 to-pink-100">
        <div className="bg-white border-2 border-red-200 rounded-xl shadow-xl p-8 max-w-md">
          <div className="text-center">
            <div className="inline-flex items-center justify-center w-16 h-16 bg-red-100 rounded-full mb-4">
              <svg className="w-8 h-8 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <h3 className="text-xl font-bold text-gray-900 mb-2">Connection Error</h3>
            <p className="text-gray-600">{error}</p>
            <button
              onClick={() => window.location.reload()}
              className="mt-6 px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
            >
              Retry
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-white">
      {/* Hero Section */}
      <div className="border-b border-gray-200 bg-white">
        <div className="max-w-7xl mx-auto px-6 lg:px-8 py-16">
          <div className="max-w-3xl">
            <h1 className="text-5xl font-bold text-[#1B025A] mb-6 leading-tight">
              ChainGuard Dashboard
            </h1>
            <p className="text-xl text-gray-600 leading-relaxed mb-8">
              Secure multi-chain AI agent management with role-based access control, threshold signatures, and comprehensive audit logging on Internet Computer Protocol.
            </p>
            <div className="flex items-center gap-3">
              <div className={`inline-flex items-center px-4 py-2 rounded-full text-sm font-semibold ${
                stats.paused
                  ? 'bg-red-50 text-red-700'
                  : 'bg-emerald-50 text-emerald-700'
              }`}>
                <div className={`w-2 h-2 rounded-full mr-2 ${
                  stats.paused ? 'bg-red-500' : 'bg-emerald-500 animate-pulse'
                }`}></div>
                {stats.paused ? 'System Paused' : 'System Operational'}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-6 lg:px-8 py-12">

        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
          <StatCard title="Pending Requests" value={stats.pendingRequests} subtitle="Awaiting signatures" />
          <StatCard title="Total Actions" value={stats.totalActions} subtitle="All time executed" />
          <StatCard title="Active Policies" value={stats.activePolicies} subtitle="Currently enforced" />
          <StatCard title="Audit Entries" value={stats.totalActions} subtitle="Complete trail" />
        </div>

        {/* Getting Started Section */}
        <div className="mb-12">
          <h2 className="text-3xl font-bold text-[#1B025A] mb-6">Getting Started</h2>
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <Link href="/strategies" className="group">
              <div className="h-full border border-gray-200 rounded-xl p-6 hover:border-[#3B00B9] hover:shadow-lg transition-all">
                <div className="w-12 h-12 rounded-lg bg-gradient-to-br from-[#3B00B9] to-[#6A5ACD] flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                </div>
                <h3 className="text-xl font-bold text-gray-900 mb-2 group-hover:text-[#3B00B9] transition-colors">
                  Automated Strategies
                </h3>
                <p className="text-gray-600 text-sm leading-relaxed">
                  Configure Dollar Cost Averaging and portfolio rebalancing with customizable parameters and schedules.
                </p>
              </div>
            </Link>

            <Link href="/signatures" className="group">
              <div className="h-full border border-gray-200 rounded-xl p-6 hover:border-[#3B00B9] hover:shadow-lg transition-all">
                <div className="w-12 h-12 rounded-lg bg-gradient-to-br from-[#ED1E79] to-[#F97316] flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                  </svg>
                </div>
                <h3 className="text-xl font-bold text-gray-900 mb-2 group-hover:text-[#3B00B9] transition-colors">
                  Threshold Signatures
                </h3>
                <p className="text-gray-600 text-sm leading-relaxed">
                  Review and approve pending multi-signature requests with role-based access control and policy enforcement.
                </p>
              </div>
            </Link>

            <Link href="/audit" className="group">
              <div className="h-full border border-gray-200 rounded-xl p-6 hover:border-[#3B00B9] hover:shadow-lg transition-all">
                <div className="w-12 h-12 rounded-lg bg-gradient-to-br from-[#18C39F] to-[#10B981] flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                </div>
                <h3 className="text-xl font-bold text-gray-900 mb-2 group-hover:text-[#3B00B9] transition-colors">
                  Audit Trail
                </h3>
                <p className="text-gray-600 text-sm leading-relaxed">
                  Complete transaction history with advanced filtering, policy evaluation results, and execution logs.
                </p>
              </div>
            </Link>
          </div>
        </div>

        {/* System Information */}
        {systemStatus && (
          <div className="mb-12">
            <h2 className="text-3xl font-bold text-[#1B025A] mb-6">System Configuration</h2>
            <div className="border border-gray-200 rounded-xl p-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <InfoRow label="Canister Name" value={systemStatus.name || 'ChainGuard SDK'} />
                <InfoRow label="Canister ID" value="foxtk-ziaaa-aaaai-atthq-cai" />
                <InfoRow label="Supported Chains" value={systemStatus.supported_chains?.join(', ') || 'Sepolia, Ethereum'} />
                <InfoRow label="Default Threshold" value={`${systemStatus.default_threshold?.required || 1}/${systemStatus.default_threshold?.total || 1}`} />
              </div>
            </div>
          </div>
        )}

        {/* Features Grid */}
        <div className="mb-12">
          <h2 className="text-3xl font-bold text-[#1B025A] mb-6">Key Features</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <FeatureCard
              title="Role-Based Access Control"
              description="Manage permissions with Owner, Operator, and Viewer roles for granular security."
            />
            <FeatureCard
              title="Policy Engine"
              description="Define custom policies with conditions and automatic enforcement for all transactions."
            />
            <FeatureCard
              title="Multi-Chain Support"
              description="Execute transactions across Ethereum, Sepolia testnet, and other EVM-compatible chains."
            />
            <FeatureCard
              title="Chain-Key ECDSA"
              description="Secure transaction signing using Internet Computer's threshold ECDSA protocol."
            />
          </div>
        </div>
      </div>
    </div>
  );
}

// Components
function StatCard({ title, value, subtitle }: any) {
  return (
    <div className="border border-gray-200 rounded-xl p-6 hover:border-[#3B00B9] hover:shadow-sm transition-all bg-white">
      <div className="text-4xl font-bold text-[#1B025A] mb-2">{value}</div>
      <div className="text-sm font-semibold text-gray-900 mb-1">{title}</div>
      <div className="text-xs text-gray-500">{subtitle}</div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="py-3">
      <dt className="text-sm font-medium text-gray-500 mb-1">{label}</dt>
      <dd className="text-base text-[#1B025A] font-mono font-semibold break-all">{value}</dd>
    </div>
  );
}

function FeatureCard({ title, description }: { title: string; description: string }) {
  return (
    <div className="border border-gray-200 rounded-xl p-6 hover:border-[#3B00B9] hover:shadow-sm transition-all bg-white">
      <h3 className="text-lg font-bold text-gray-900 mb-2">{title}</h3>
      <p className="text-sm text-gray-600 leading-relaxed">{description}</p>
    </div>
  );
}
