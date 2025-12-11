'use client';

import { useEffect, useState } from 'react';
import { useChainGuard } from '@/lib/hooks/useChainGuard';
import type { PendingRequest } from '@/lib/types/chainguard';

export default function SignaturesPage() {
  const { getPendingRequests, signRequest, loading, error, principal } = useChainGuard();
  const [pendingRequests, setPendingRequests] = useState<PendingRequest[]>([]);
  const [selectedRequest, setSelectedRequest] = useState<PendingRequest | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  useEffect(() => {
    loadPendingRequests();
  }, [loading]);

  async function loadPendingRequests() {
    if (!loading) {
      setRefreshing(true);
      const requests = await getPendingRequests();
      setPendingRequests(requests);
      setRefreshing(false);
    }
  }

  async function handleSign(requestId: bigint) {
    const result = await signRequest(requestId);
    if (result) {
      console.log('Signature added:', result);
      await loadPendingRequests();
    }
  }

  function formatTimestamp(timestamp: bigint): string {
    const date = new Date(Number(timestamp) / 1000000);
    return date.toLocaleString();
  }

  function formatPrincipal(p: any): string {
    const str = p?.toText ? p.toText() : String(p);
    return str.slice(0, 10) + '...' + str.slice(-8);
  }

  function getActionDescription(action: any): string {
    if ('Transfer' in action) {
      const t = action.Transfer;
      return `Transfer ${Number(t.amount) / 1e18} ${t.token} to ${t.to.slice(0, 10)}...`;
    }
    if ('Swap' in action) {
      const s = action.Swap;
      return `Swap ${Number(s.amount_in) / 1e6} ${s.token_in.slice(0, 6)}... → ${s.token_out}`;
    }
    if ('ApproveToken' in action) {
      const a = action.ApproveToken;
      return `Approve ${Number(a.amount) / 1e18} ${a.token} for ${a.spender.slice(0, 10)}...`;
    }
    return 'Unknown action';
  }

  function getStatusColor(status: any): string {
    if ('Pending' in status) return 'bg-[#F59E0B]/10 text-[#F59E0B]';
    if ('Approved' in status) return 'bg-[#3B00B9]/10 text-[#3B00B9]';
    if ('Executed' in status) return 'bg-[#18C39F]/10 text-[#18C39F]';
    if ('Expired' in status) return 'bg-gray-100 text-gray-600';
    if ('Rejected' in status) return 'bg-[#EF4444]/10 text-[#EF4444]';
    return 'bg-gray-100 text-gray-700';
  }

  function getStatusText(status: any): string {
    if ('Pending' in status) return 'Pending';
    if ('Approved' in status) return 'Approved';
    if ('Executed' in status) return 'Executed';
    if ('Expired' in status) return 'Expired';
    if ('Rejected' in status) return 'Rejected';
    return 'Unknown';
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-4 border-blue-600 mx-auto"></div>
          <p className="mt-4 text-gray-700 font-medium">Loading pending signatures...</p>
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

  return (
    <div className="min-h-screen bg-white">
      {/* Hero Section */}
      <div className="border-b border-gray-200 bg-white">
        <div className="max-w-7xl mx-auto px-6 lg:px-8 py-16">
          <div className="flex justify-between items-start">
            <div className="max-w-3xl">
              <h1 className="text-5xl font-bold text-[#1B025A] mb-6 leading-tight">
                Threshold Signatures
              </h1>
              <p className="text-xl text-gray-600 leading-relaxed mb-8">
                Review and approve pending multi-signature requests with role-based access control and policy enforcement. Ensure secure transaction execution through distributed signature collection.
              </p>
            </div>
            <button
              onClick={loadPendingRequests}
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

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-6 lg:px-8 py-12">

        {/* Stats Grid */}
        <div className="mb-12">
          <h2 className="text-3xl font-bold text-[#1B025A] mb-6">Overview</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div className="border border-gray-200 rounded-xl p-6 bg-white hover:border-[#3B00B9] hover:shadow-sm transition-all">
              <div className="text-4xl font-bold text-[#1B025A] mb-2">{pendingRequests.length}</div>
              <div className="text-sm font-semibold text-gray-900 mb-1">Total Pending</div>
              <div className="text-xs text-gray-500">Requests awaiting signatures</div>
            </div>
            <div className="border border-gray-200 rounded-xl p-6 bg-white hover:border-[#3B00B9] hover:shadow-sm transition-all">
              <div className="text-4xl font-bold text-[#ED1E79] mb-2">
                {pendingRequests.filter(r =>
                  !r.collected_signatures.some(s => s.signer.toText() === principal?.toText())
                ).length}
              </div>
              <div className="text-sm font-semibold text-gray-900 mb-1">Awaiting My Signature</div>
              <div className="text-xs text-gray-500">Requests you haven't signed</div>
            </div>
            <div className="border border-gray-200 rounded-xl p-6 bg-white hover:border-[#3B00B9] hover:shadow-sm transition-all">
              <div className="text-4xl font-bold text-[#3B00B9] mb-2">
                {pendingRequests.filter(r => 'Transfer' in r.action).length}
              </div>
              <div className="text-sm font-semibold text-gray-900 mb-1">Transfers</div>
              <div className="text-xs text-gray-500">Token transfer requests</div>
            </div>
            <div className="border border-gray-200 rounded-xl p-6 bg-white hover:border-[#3B00B9] hover:shadow-sm transition-all">
              <div className="text-4xl font-bold text-[#6A5ACD] mb-2">
                {pendingRequests.filter(r => 'Swap' in r.action).length}
              </div>
              <div className="text-sm font-semibold text-gray-900 mb-1">Swaps</div>
              <div className="text-xs text-gray-500">Token swap requests</div>
            </div>
          </div>
        </div>

        {/* Requests Table */}
        <div>
          <h2 className="text-3xl font-bold text-[#1B025A] mb-6">Pending Requests</h2>

          {pendingRequests.length === 0 ? (
            <div className="border border-gray-200 rounded-xl p-12 text-center bg-white">
              <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <h3 className="text-xl font-bold text-gray-900 mb-2">No Pending Requests</h3>
              <p className="text-gray-600">All requests have been processed or there are no pending signatures.</p>
            </div>
          ) : (
            <div className="border border-gray-200 rounded-xl overflow-hidden bg-white">
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead className="bg-gray-50 border-b border-gray-200">
                    <tr>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">ID</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Action</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Requester</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Created</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Signatures</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Status</th>
                      <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {pendingRequests.map((request) => {
                      const hasSignedByMe = request.collected_signatures.some(
                        s => s.signer.toText() === principal?.toText()
                      );

                      return (
                        <tr key={request.id.toString()} className="border-b border-gray-100 hover:bg-gray-50 transition-colors">
                          <td className="py-4 px-6">
                            <span className="font-mono text-sm font-semibold text-gray-900">#{request.id.toString()}</span>
                          </td>
                          <td className="py-4 px-6">
                            <div className="text-sm text-gray-900 font-medium">{getActionDescription(request.action)}</div>
                          </td>
                          <td className="py-4 px-6">
                            <span className="font-mono text-xs text-gray-600">{formatPrincipal(request.requester)}</span>
                          </td>
                          <td className="py-4 px-6">
                            <span className="text-sm text-gray-600">{formatTimestamp(request.created_at)}</span>
                          </td>
                          <td className="py-4 px-6">
                            <div className="flex items-center gap-2">
                              <div className="text-sm font-bold text-gray-900">
                                {request.collected_signatures.length}/{request.required_signatures}
                              </div>
                              <div className="flex-1 bg-gray-200 rounded-full h-2 min-w-[60px]">
                                <div
                                  className="bg-[#3B00B9] h-2 rounded-full transition-all"
                                  style={{
                                    width: `${(request.collected_signatures.length / request.required_signatures) * 100}%`
                                  }}
                                ></div>
                              </div>
                            </div>
                          </td>
                          <td className="py-4 px-6">
                            <span className={`inline-block px-3 py-1 rounded-full text-xs font-semibold ${getStatusColor(request.status)}`}>
                              {getStatusText(request.status)}
                            </span>
                          </td>
                          <td className="py-4 px-6">
                            <div className="flex gap-2">
                              <button
                                onClick={() => setSelectedRequest(request)}
                                className="px-3 py-1.5 text-sm font-medium text-[#3B00B9] hover:bg-[#3B00B9]/5 rounded-lg transition border border-gray-200"
                              >
                                View
                              </button>
                              {!hasSignedByMe && 'Pending' in request.status && (
                                <button
                                  onClick={() => handleSign(request.id)}
                                  className="px-3 py-1.5 text-sm font-medium bg-[#18C39F] text-white hover:bg-[#10B981] rounded-lg transition"
                                >
                                  Sign
                                </button>
                              )}
                              {hasSignedByMe && (
                                <span className="px-3 py-1.5 text-sm font-medium text-[#18C39F] bg-[#18C39F]/10 rounded-lg border border-[#18C39F]/20">
                                  Signed
                                </span>
                              )}
                            </div>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Request Details Modal */}
      {selectedRequest && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-xl shadow-2xl max-w-3xl w-full max-h-[85vh] overflow-y-auto border border-gray-200">
            {/* Modal Header */}
            <div className="p-6 border-b border-gray-200 bg-gray-50">
              <div className="flex justify-between items-start">
                <div>
                  <h2 className="text-2xl font-bold text-[#1B025A]">Request Details</h2>
                  <p className="text-sm text-gray-600 mt-1 font-mono">ID: #{selectedRequest.id.toString()}</p>
                </div>
                <button
                  onClick={() => setSelectedRequest(null)}
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
              {/* Action Details */}
              <div>
                <h3 className="text-lg font-bold text-gray-900 mb-3">Action Details</h3>
                <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                  <pre className="font-mono text-xs text-gray-900 whitespace-pre-wrap overflow-x-auto">
                    {JSON.stringify(selectedRequest.action, null, 2)}
                  </pre>
                </div>
              </div>

              {/* Request Information */}
              <div>
                <h3 className="text-lg font-bold text-gray-900 mb-3">Request Information</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div className="border border-gray-200 rounded-lg p-4 bg-white">
                    <div className="text-xs font-semibold text-gray-500 mb-2">Requester</div>
                    <div className="text-xs font-mono text-gray-900 break-all">
                      {selectedRequest.requester.toText()}
                    </div>
                  </div>
                  <div className="border border-gray-200 rounded-lg p-4 bg-white">
                    <div className="text-xs font-semibold text-gray-500 mb-2">Status</div>
                    <span className={`inline-block px-3 py-1 rounded-full text-xs font-semibold ${getStatusColor(selectedRequest.status)}`}>
                      {getStatusText(selectedRequest.status)}
                    </span>
                  </div>
                  <div className="border border-gray-200 rounded-lg p-4 bg-white">
                    <div className="text-xs font-semibold text-gray-500 mb-2">Created</div>
                    <div className="text-sm font-medium text-gray-900">{formatTimestamp(selectedRequest.created_at)}</div>
                  </div>
                  <div className="border border-gray-200 rounded-lg p-4 bg-white">
                    <div className="text-xs font-semibold text-gray-500 mb-2">Expires</div>
                    <div className="text-sm font-medium text-gray-900">{formatTimestamp(selectedRequest.expires_at)}</div>
                  </div>
                </div>
              </div>

              {/* Signatures */}
              <div>
                <h3 className="text-lg font-bold text-gray-900 mb-3">
                  Collected Signatures ({selectedRequest.collected_signatures.length}/{selectedRequest.required_signatures})
                </h3>
                <div className="space-y-2">
                  {selectedRequest.collected_signatures.map((sig, idx) => (
                    <div key={idx} className="flex items-center justify-between border border-[#18C39F]/20 bg-[#18C39F]/5 rounded-lg p-4">
                      <div className="flex-1">
                        <div className="text-xs font-semibold text-[#18C39F] mb-1">Signer {idx + 1}</div>
                        <div className="text-xs font-mono text-gray-600 break-all">{sig.signer.toText()}</div>
                      </div>
                      <div className="text-xs text-gray-600 font-medium ml-4 text-right">
                        {formatTimestamp(sig.signed_at)}
                      </div>
                    </div>
                  ))}
                  {selectedRequest.collected_signatures.length < selectedRequest.required_signatures && (
                    <div className="border border-[#F59E0B]/20 bg-[#F59E0B]/5 rounded-lg p-4 text-center">
                      <div className="text-sm text-[#F59E0B] font-semibold">
                        Waiting for {selectedRequest.required_signatures - selectedRequest.collected_signatures.length} more signature(s)
                      </div>
                    </div>
                  )}
                </div>
              </div>

              {/* Action Buttons */}
              <div className="flex gap-3 pt-4">
                <button
                  onClick={() => {
                    handleSign(selectedRequest.id);
                    setSelectedRequest(null);
                  }}
                  disabled={selectedRequest.collected_signatures.some(s => s.signer.toText() === principal?.toText())}
                  className="flex-1 bg-[#18C39F] text-white font-semibold py-3 px-6 rounded-lg hover:bg-[#10B981] transition disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Sign Request
                </button>
                <button
                  onClick={() => setSelectedRequest(null)}
                  className="px-6 py-3 border-2 border-gray-200 text-gray-700 font-semibold rounded-lg hover:bg-gray-50 transition"
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
