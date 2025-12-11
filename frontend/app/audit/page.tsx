'use client';

import { useEffect, useState } from 'react';
import { useChainGuard } from '@/lib/hooks/useChainGuard';
import type { AuditEntry } from '@/lib/types/chainguard';

export default function AuditPage() {
  const { getAuditLogs, loading, error } = useChainGuard();
  const [auditLogs, setAuditLogs] = useState<AuditEntry[]>([]);
  const [filteredLogs, setFilteredLogs] = useState<AuditEntry[]>([]);
  const [selectedEntry, setSelectedEntry] = useState<AuditEntry | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  // Filters
  const [filters, setFilters] = useState({
    actionType: 'all',
    decision: 'all',
    dateFrom: '',
    dateTo: '',
    searchText: '',
  });

  useEffect(() => {
    loadAuditLogs();
  }, [loading]);

  useEffect(() => {
    applyFilters();
  }, [auditLogs, filters]);

  async function loadAuditLogs() {
    if (!loading) {
      setRefreshing(true);
      const logs = await getAuditLogs();
      setAuditLogs(logs);
      setRefreshing(false);
    }
  }

  function applyFilters() {
    let filtered = [...auditLogs];

    // Filter by action type
    if (filters.actionType !== 'all') {
      filtered = filtered.filter(log => log.action_type === filters.actionType);
    }

    // Filter by decision
    if (filters.decision !== 'all') {
      filtered = filtered.filter(log => {
        if (filters.decision === 'Allowed' && 'Allowed' in log.policy_result.decision) return true;
        if (filters.decision === 'Denied' && 'Denied' in log.policy_result.decision) return true;
        if (filters.decision === 'RequiresThreshold' && 'RequiresThreshold' in log.policy_result.decision) return true;
        return false;
      });
    }

    // Filter by search text
    if (filters.searchText) {
      const search = filters.searchText.toLowerCase();
      filtered = filtered.filter(log =>
        log.action_type.toLowerCase().includes(search) ||
        log.action_params.toLowerCase().includes(search) ||
        log.policy_result.reason.toLowerCase().includes(search)
      );
    }

    setFilteredLogs(filtered);
  }

  function formatTimestamp(timestamp: bigint): string {
    const date = new Date(Number(timestamp) / 1000000);
    return date.toLocaleString();
  }

  function formatPrincipal(p: any): string {
    const str = p?.toText ? p.toText() : String(p);
    return str.slice(0, 10) + '...' + str.slice(-8);
  }

  function getDecisionColor(decision: any): string {
    if ('Allowed' in decision) return 'bg-[#18C39F]/10 text-[#18C39F]';
    if ('Denied' in decision) return 'bg-[#EF4444]/10 text-[#EF4444]';
    if ('RequiresThreshold' in decision) return 'bg-[#F59E0B]/10 text-[#F59E0B]';
    return 'bg-gray-100 text-gray-700';
  }

  function getDecisionText(decision: any): string {
    if ('Allowed' in decision) return 'Allowed';
    if ('Denied' in decision) return 'Denied';
    if ('RequiresThreshold' in decision) return 'Requires Threshold';
    return 'Unknown';
  }

  function getActionTypeColor(actionType: string): string {
    if (actionType === 'Transfer') return 'bg-[#3B00B9]/10 text-[#3B00B9]';
    if (actionType === 'Swap') return 'bg-[#ED1E79]/10 text-[#ED1E79]';
    if (actionType === 'ApproveToken') return 'bg-[#6A5ACD]/10 text-[#6A5ACD]';
    return 'bg-gray-100 text-gray-700';
  }

  function exportToCSV() {
    const headers = ['ID', 'Timestamp', 'Action Type', 'Decision', 'Requester', 'Reason', 'TX Hash'];
    const rows = filteredLogs.map(log => [
      log.id.toString(),
      formatTimestamp(log.timestamp),
      log.action_type,
      getDecisionText(log.policy_result.decision),
      log.requester.toText(),
      log.policy_result.reason,
      log.execution_result && log.execution_result[0]?.tx_hash ? log.execution_result[0].tx_hash[0] || '' : ''
    ]);

    const csv = [headers, ...rows].map(row => row.map(cell => `"${cell}"`).join(',')).join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `audit-logs-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-4 border-blue-600 mx-auto"></div>
          <p className="mt-4 text-gray-700 font-medium">Loading audit logs...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gradient-to-br from-red-50 to-pink-100">
        <div className="bg-white border-2 border-red-200 rounded-xl shadow-xl p-8 max-w-md">
          <div className="text-center">
            <h3 className="text-xl font-bold text-gray-900 mb-2">Connection Error</h3>
            <p className="text-gray-600">{error}</p>
          </div>
        </div>
      </div>
    );
  }

  const allowedCount = auditLogs.filter(log => 'Allowed' in log.policy_result.decision).length;
  const deniedCount = auditLogs.filter(log => 'Denied' in log.policy_result.decision).length;
  const thresholdCount = auditLogs.filter(log => 'RequiresThreshold' in log.policy_result.decision).length;

  return (
    <div className="min-h-screen bg-white">
      {/* Hero Section */}
      <div className="border-b border-gray-200 bg-white">
        <div className="max-w-7xl mx-auto px-6 lg:px-8 py-16">
          <div className="flex justify-between items-start">
            <div className="max-w-3xl">
              <h1 className="text-5xl font-bold text-[#1B025A] mb-6 leading-tight">
                Audit Trail
              </h1>
              <p className="text-xl text-gray-600 leading-relaxed mb-8">
                Complete transaction history with advanced filtering, policy evaluation results, and execution logs. Monitor all system activity with comprehensive audit tracking and compliance reporting.
              </p>
            </div>
            <div className="flex gap-3">
              <button
                onClick={exportToCSV}
                disabled={filteredLogs.length === 0}
                className="flex items-center gap-2 px-4 py-3 bg-[#18C39F] text-white rounded-lg hover:bg-[#10B981] transition disabled:opacity-50 font-semibold"
              >
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                Export CSV
              </button>
              <button
                onClick={loadAuditLogs}
                disabled={refreshing}
                className="flex items-center gap-2 px-4 py-3 bg-[#3B00B9] text-white rounded-lg hover:bg-[#1B025A] transition disabled:opacity-50 font-semibold"
              >
                <svg className={`w-5 h-5 ${refreshing ? 'animate-spin' : ''}`} fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
                Refresh
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-6 lg:px-8 py-12">

        {/* Stats Grid */}
        <div className="mb-12">
          <h2 className="text-3xl font-bold text-[#1B025A] mb-6">Activity Overview</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div className="border border-gray-200 rounded-xl p-6 bg-white hover:border-[#3B00B9] hover:shadow-sm transition-all">
              <div className="text-4xl font-bold text-[#1B025A] mb-2">{auditLogs.length}</div>
              <div className="text-sm font-semibold text-gray-900 mb-1">Total Logs</div>
              <div className="text-xs text-gray-500">All audit entries</div>
            </div>
            <div className="border border-gray-200 rounded-xl p-6 bg-white hover:border-[#3B00B9] hover:shadow-sm transition-all">
              <div className="text-4xl font-bold text-[#18C39F] mb-2">{allowedCount}</div>
              <div className="text-sm font-semibold text-gray-900 mb-1">Allowed</div>
              <div className="text-xs text-gray-500">Policy approved</div>
            </div>
            <div className="border border-gray-200 rounded-xl p-6 bg-white hover:border-[#3B00B9] hover:shadow-sm transition-all">
              <div className="text-4xl font-bold text-[#EF4444] mb-2">{deniedCount}</div>
              <div className="text-sm font-semibold text-gray-900 mb-1">Denied</div>
              <div className="text-xs text-gray-500">Policy blocked</div>
            </div>
            <div className="border border-gray-200 rounded-xl p-6 bg-white hover:border-[#3B00B9] hover:shadow-sm transition-all">
              <div className="text-4xl font-bold text-[#F59E0B] mb-2">{thresholdCount}</div>
              <div className="text-sm font-semibold text-gray-900 mb-1">Threshold</div>
              <div className="text-xs text-gray-500">Requires signatures</div>
            </div>
          </div>
        </div>

        {/* Filters */}
        <div className="mb-8">
          <h2 className="text-3xl font-bold text-[#1B025A] mb-6">Filter Logs</h2>
          <div className="border border-gray-200 rounded-xl p-6 bg-white">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
              <div>
                <label className="block text-sm font-semibold text-gray-900 mb-2">Action Type</label>
                <select
                  value={filters.actionType}
                  onChange={(e) => setFilters({ ...filters, actionType: e.target.value })}
                  className="w-full px-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-[#3B00B9] focus:border-transparent bg-white text-gray-900"
                >
                  <option value="all">All Types</option>
                  <option value="Transfer">Transfer</option>
                  <option value="Swap">Swap</option>
                  <option value="ApproveToken">Approve Token</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-900 mb-2">Decision</label>
                <select
                  value={filters.decision}
                  onChange={(e) => setFilters({ ...filters, decision: e.target.value })}
                  className="w-full px-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-[#3B00B9] focus:border-transparent bg-white text-gray-900"
                >
                  <option value="all">All Decisions</option>
                  <option value="Allowed">Allowed</option>
                  <option value="Denied">Denied</option>
                  <option value="RequiresThreshold">Requires Threshold</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-900 mb-2">Search</label>
                <input
                  type="text"
                  value={filters.searchText}
                  onChange={(e) => setFilters({ ...filters, searchText: e.target.value })}
                  placeholder="Search logs..."
                  className="w-full px-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-[#3B00B9] focus:border-transparent bg-white text-gray-900"
                />
              </div>

              <div className="flex items-end">
                <button
                  onClick={() => setFilters({ actionType: 'all', decision: 'all', dateFrom: '', dateTo: '', searchText: '' })}
                  className="w-full px-4 py-3 bg-gray-100 text-gray-700 font-semibold rounded-lg hover:bg-gray-200 transition border border-gray-200"
                >
                  Clear Filters
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* Logs Table */}
        <div>
          <h2 className="text-3xl font-bold text-[#1B025A] mb-6">
            Audit Entries {filteredLogs.length !== auditLogs.length && `(${filteredLogs.length} of ${auditLogs.length})`}
          </h2>

          {filteredLogs.length === 0 ? (
            <div className="border border-gray-200 rounded-xl p-12 text-center bg-white">
              <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <h3 className="text-xl font-bold text-gray-900 mb-2">No Audit Logs Found</h3>
              <p className="text-gray-600">Try adjusting your filters or there are no logs yet.</p>
            </div>
          ) : (
            <div className="border border-gray-200 rounded-xl overflow-hidden bg-white">
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead className="bg-gray-50 border-b border-gray-200">
                    <tr>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">ID</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Timestamp</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Type</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Requester</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Decision</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Result</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Details</th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredLogs.map((log) => (
                      <tr key={log.id.toString()} className="border-b border-gray-100 hover:bg-gray-50 transition-colors">
                        <td className="py-4 px-6">
                          <span className="font-mono text-sm font-semibold text-gray-900">#{log.id.toString()}</span>
                        </td>
                        <td className="py-4 px-6">
                          <span className="text-sm text-gray-600">{formatTimestamp(log.timestamp)}</span>
                        </td>
                        <td className="py-4 px-6">
                          <span className={`inline-block px-3 py-1 rounded-full text-xs font-semibold ${getActionTypeColor(log.action_type)}`}>
                            {log.action_type}
                          </span>
                        </td>
                        <td className="py-4 px-6">
                          <span className="font-mono text-xs text-gray-600">{formatPrincipal(log.requester)}</span>
                        </td>
                        <td className="py-4 px-6">
                          <span className={`inline-block px-3 py-1 rounded-full text-xs font-semibold ${getDecisionColor(log.policy_result.decision)}`}>
                            {getDecisionText(log.policy_result.decision)}
                          </span>
                        </td>
                        <td className="py-4 px-6">
                          {log.execution_result && log.execution_result[0] ? (
                            <div>
                              {log.execution_result[0].success ? (
                                <span className="inline-block px-3 py-1 rounded-full text-xs font-semibold bg-[#18C39F]/10 text-[#18C39F]">
                                  Success
                                </span>
                              ) : (
                                <span className="inline-block px-3 py-1 rounded-full text-xs font-semibold bg-[#EF4444]/10 text-[#EF4444]">
                                  Failed
                                </span>
                              )}
                            </div>
                          ) : (
                            <span className="text-gray-400 text-xs">N/A</span>
                          )}
                        </td>
                        <td className="py-4 px-6">
                          <button
                            onClick={() => setSelectedEntry(log)}
                            className="px-3 py-1.5 text-sm font-medium text-[#3B00B9] hover:bg-[#3B00B9]/5 rounded-lg transition border border-gray-200"
                          >
                            View
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Entry Details Modal */}
      {selectedEntry && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-xl shadow-2xl max-w-4xl w-full max-h-[85vh] overflow-y-auto border border-gray-200">
            {/* Modal Header */}
            <div className="p-6 border-b border-gray-200 bg-gray-50">
              <div className="flex justify-between items-start">
                <div>
                  <h2 className="text-2xl font-bold text-[#1B025A]">Audit Entry Details</h2>
                  <p className="text-sm text-gray-600 mt-1 font-mono">ID: #{selectedEntry.id.toString()}</p>
                </div>
                <button
                  onClick={() => setSelectedEntry(null)}
                  className="text-gray-400 hover:text-gray-600 p-2 hover:bg-gray-200 rounded-lg transition"
                >
                  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>

            {/* Modal Content */}
            <div className="p-6 space-y-6">
              {/* Basic Info */}
              <div>
                <h3 className="text-lg font-bold text-gray-900 mb-3">Basic Information</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div className="border border-gray-200 rounded-lg p-4 bg-white">
                    <div className="text-xs font-semibold text-gray-500 mb-2">Timestamp</div>
                    <div className="text-sm font-medium text-gray-900">{formatTimestamp(selectedEntry.timestamp)}</div>
                  </div>
                  <div className="border border-gray-200 rounded-lg p-4 bg-white">
                    <div className="text-xs font-semibold text-gray-500 mb-2">Action Type</div>
                    <span className={`inline-block px-3 py-1 rounded-full text-xs font-semibold ${getActionTypeColor(selectedEntry.action_type)}`}>
                      {selectedEntry.action_type}
                    </span>
                  </div>
                  <div className="border border-gray-200 rounded-lg p-4 bg-white col-span-2">
                    <div className="text-xs font-semibold text-gray-500 mb-2">Requester</div>
                    <div className="text-xs font-mono text-gray-900 break-all">{selectedEntry.requester.toText()}</div>
                  </div>
                </div>
              </div>

              {/* Action Parameters */}
              <div>
                <h3 className="text-lg font-bold text-gray-900 mb-3">Action Parameters</h3>
                <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                  <pre className="font-mono text-xs text-gray-900 whitespace-pre-wrap break-all overflow-x-auto">
                    {selectedEntry.action_params}
                  </pre>
                </div>
              </div>

              {/* Policy Result */}
              <div>
                <h3 className="text-lg font-bold text-gray-900 mb-3">Policy Evaluation</h3>
                <div className="space-y-3">
                  <div className="flex items-center gap-3">
                    <span className="text-sm font-semibold text-gray-600">Decision:</span>
                    <span className={`inline-block px-3 py-1 rounded-full text-xs font-semibold ${getDecisionColor(selectedEntry.policy_result.decision)}`}>
                      {getDecisionText(selectedEntry.policy_result.decision)}
                    </span>
                  </div>
                  {selectedEntry.policy_result.matched_policy[0] && (
                    <div className="border border-[#3B00B9]/20 bg-[#3B00B9]/5 rounded-lg p-4">
                      <div className="text-xs font-semibold text-[#3B00B9] mb-2">Matched Policy</div>
                      <div className="text-sm text-gray-900">{selectedEntry.policy_result.matched_policy[0]}</div>
                    </div>
                  )}
                  <div className="border border-gray-200 rounded-lg p-4 bg-white">
                    <div className="text-xs font-semibold text-gray-500 mb-2">Reason</div>
                    <div className="text-sm text-gray-900">{selectedEntry.policy_result.reason}</div>
                  </div>
                </div>
              </div>

              {/* Execution Result */}
              {selectedEntry.execution_result && selectedEntry.execution_result[0] && (
                <div>
                  <h3 className="text-lg font-bold text-gray-900 mb-3">Execution Result</h3>
                  <div className="space-y-3">
                    <div className="flex items-center gap-3">
                      <span className="text-sm font-semibold text-gray-600">Status:</span>
                      {selectedEntry.execution_result[0].success ? (
                        <span className="inline-block px-3 py-1 rounded-full text-xs font-semibold bg-[#18C39F]/10 text-[#18C39F]">
                          Success
                        </span>
                      ) : (
                        <span className="inline-block px-3 py-1 rounded-full text-xs font-semibold bg-[#EF4444]/10 text-[#EF4444]">
                          Failed
                        </span>
                      )}
                    </div>
                    <div className="border border-gray-200 rounded-lg p-4 bg-white">
                      <div className="text-xs font-semibold text-gray-500 mb-2">Chain</div>
                      <div className="text-sm font-medium text-gray-900">{selectedEntry.execution_result[0].chain}</div>
                    </div>
                    {selectedEntry.execution_result[0].tx_hash[0] && (
                      <div className="border border-[#18C39F]/20 bg-[#18C39F]/5 rounded-lg p-4">
                        <div className="text-xs font-semibold text-[#18C39F] mb-2">Transaction Hash</div>
                        <a
                          href={`https://sepolia.etherscan.io/tx/${selectedEntry.execution_result[0].tx_hash[0]}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-sm font-mono text-[#3B00B9] hover:text-[#1B025A] break-all hover:underline"
                        >
                          {selectedEntry.execution_result[0].tx_hash[0]}
                        </a>
                      </div>
                    )}
                    {selectedEntry.execution_result[0].error[0] && (
                      <div className="border border-[#EF4444]/20 bg-[#EF4444]/5 rounded-lg p-4">
                        <div className="text-xs font-semibold text-[#EF4444] mb-2">Error</div>
                        <div className="text-sm text-gray-900">{selectedEntry.execution_result[0].error[0]}</div>
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* Threshold Request ID */}
              {selectedEntry.threshold_request_id[0] && (
                <div className="border border-[#F59E0B]/20 bg-[#F59E0B]/5 rounded-lg p-4">
                  <div className="text-xs font-semibold text-[#F59E0B] mb-2">Threshold Request ID</div>
                  <div className="text-sm font-mono text-gray-900">{selectedEntry.threshold_request_id[0].toString()}</div>
                </div>
              )}

              {/* Close Button */}
              <div className="pt-4">
                <button
                  onClick={() => setSelectedEntry(null)}
                  className="w-full px-6 py-3 border-2 border-gray-200 text-gray-700 font-semibold rounded-lg hover:bg-gray-50 transition"
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
