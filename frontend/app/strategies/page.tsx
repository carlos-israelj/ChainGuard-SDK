'use client';

import { useState } from 'react';
import { useChainGuard } from '@/lib/hooks/useChainGuard';

export default function StrategiesPage() {
  const { requestAction, loading, error } = useChainGuard();
  const [activeStrategy, setActiveStrategy] = useState<'dca' | 'rebalance'>('dca');

  // DCA State
  const [dcaConfig, setDcaConfig] = useState({
    chain: 'Sepolia',
    sourceToken: 'USDC',
    targetToken: 'ETH',
    amountPerPurchase: '1000000',
    interval: 'daily',
    enabled: true,
  });

  // Rebalancing State
  const [rebalanceConfig, setRebalanceConfig] = useState({
    chain: 'Sepolia',
    portfolio: [
      { token: 'ETH', targetPercent: 50, currentPercent: 55 },
      { token: 'USDC', targetPercent: 30, currentPercent: 25 },
      { token: 'WBTC', targetPercent: 20, currentPercent: 20 },
    ],
    rebalanceThreshold: 5,
    enabled: true,
  });

  const [executionHistory, setExecutionHistory] = useState([
    { id: 1, timestamp: '2025-12-11 10:30:00', strategy: 'DCA', action: 'Swap 1000 USDC → ETH', status: 'Success', txHash: '0xfd8d8b...' },
    { id: 2, timestamp: '2025-12-10 10:30:00', strategy: 'DCA', action: 'Swap 1000 USDC → ETH', status: 'Success', txHash: '0x7fa2b1...' },
    { id: 3, timestamp: '2025-12-09 14:15:00', strategy: 'Rebalance', action: 'Portfolio rebalance', status: 'Success', txHash: '0x3c4d5e...' },
    { id: 4, timestamp: '2025-12-09 10:30:00', strategy: 'DCA', action: 'Swap 1000 USDC → ETH', status: 'Pending Signatures', txHash: null },
  ]);

  const handleDcaSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    console.log('DCA Configuration:', dcaConfig);
    // In production, this would call requestAction with a Swap action
  };

  const handleRebalanceSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    console.log('Rebalance Configuration:', rebalanceConfig);
    // In production, this would trigger the rebalancing logic
  };

  const handleExecuteNow = async () => {
    if (activeStrategy === 'dca') {
      console.log('Executing DCA now...');
      // Call requestAction with Swap
    } else {
      console.log('Executing Rebalance now...');
      // Call rebalancing logic
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-4 border-blue-600 mx-auto"></div>
          <p className="mt-4 text-gray-700 font-medium">Loading strategies...</p>
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
          <div className="max-w-3xl">
            <h1 className="text-5xl font-bold text-[#1B025A] mb-6 leading-tight">
              Automated Strategies
            </h1>
            <p className="text-xl text-gray-600 leading-relaxed mb-8">
              Configure and monitor Dollar Cost Averaging (DCA) and portfolio rebalancing strategies with customizable parameters, automated execution schedules, and comprehensive performance tracking.
            </p>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-6 lg:px-8 py-12">

        {/* Strategy Selector */}
        <div className="mb-12">
          <h2 className="text-3xl font-bold text-[#1B025A] mb-6">Select Strategy</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <button
              onClick={() => setActiveStrategy('dca')}
              className={`text-left border rounded-xl p-6 transition-all ${
                activeStrategy === 'dca'
                  ? 'border-[#3B00B9] bg-gradient-to-br from-[#3B00B9]/5 to-[#6A5ACD]/5 shadow-sm'
                  : 'border-gray-200 hover:border-[#3B00B9] hover:shadow-sm'
              }`}
            >
              <div className="flex items-start gap-4">
                <div className={`p-3 rounded-lg ${
                  activeStrategy === 'dca'
                    ? 'bg-gradient-to-br from-[#3B00B9] to-[#6A5ACD]'
                    : 'bg-gray-100'
                }`}>
                  <svg className={`w-6 h-6 ${activeStrategy === 'dca' ? 'text-white' : 'text-gray-600'}`} fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-xl font-bold text-gray-900 mb-2">Dollar Cost Averaging</h3>
                  <p className="text-sm text-gray-600 leading-relaxed">
                    Automatically purchase crypto assets at regular intervals to reduce volatility impact and average out entry prices over time.
                  </p>
                </div>
              </div>
            </button>

            <button
              onClick={() => setActiveStrategy('rebalance')}
              className={`text-left border rounded-xl p-6 transition-all ${
                activeStrategy === 'rebalance'
                  ? 'border-[#3B00B9] bg-gradient-to-br from-[#3B00B9]/5 to-[#6A5ACD]/5 shadow-sm'
                  : 'border-gray-200 hover:border-[#3B00B9] hover:shadow-sm'
              }`}
            >
              <div className="flex items-start gap-4">
                <div className={`p-3 rounded-lg ${
                  activeStrategy === 'rebalance'
                    ? 'bg-gradient-to-br from-[#3B00B9] to-[#6A5ACD]'
                    : 'bg-gray-100'
                }`}>
                  <svg className={`w-6 h-6 ${activeStrategy === 'rebalance' ? 'text-white' : 'text-gray-600'}`} fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-xl font-bold text-gray-900 mb-2">Portfolio Rebalancing</h3>
                  <p className="text-sm text-gray-600 leading-relaxed">
                    Maintain target asset allocations by automatically rebalancing your portfolio when deviations exceed threshold parameters.
                  </p>
                </div>
              </div>
            </button>
          </div>
        </div>

        {/* Configuration Section */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 mb-12">
          {/* Configuration Panel */}
          <div className="lg:col-span-2">
            <h2 className="text-3xl font-bold text-[#1B025A] mb-6">
              {activeStrategy === 'dca' ? 'DCA Configuration' : 'Rebalancing Configuration'}
            </h2>

            <div className="border border-gray-200 rounded-xl p-6 bg-white">
              {activeStrategy === 'dca' ? (
                <form onSubmit={handleDcaSubmit} className="space-y-6">
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                      <label className="block text-sm font-semibold text-gray-900 mb-2">Chain</label>
                      <select
                        value={dcaConfig.chain}
                        onChange={(e) => setDcaConfig({ ...dcaConfig, chain: e.target.value })}
                        className="w-full px-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-[#3B00B9] focus:border-transparent bg-white text-gray-900"
                      >
                        <option value="Sepolia">Sepolia Testnet</option>
                        <option value="Ethereum">Ethereum Mainnet</option>
                      </select>
                    </div>

                    <div>
                      <label className="block text-sm font-semibold text-gray-900 mb-2">Interval</label>
                      <select
                        value={dcaConfig.interval}
                        onChange={(e) => setDcaConfig({ ...dcaConfig, interval: e.target.value })}
                        className="w-full px-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-[#3B00B9] focus:border-transparent bg-white text-gray-900"
                      >
                        <option value="hourly">Hourly</option>
                        <option value="daily">Daily</option>
                        <option value="weekly">Weekly</option>
                        <option value="monthly">Monthly</option>
                      </select>
                    </div>

                    <div>
                      <label className="block text-sm font-semibold text-gray-900 mb-2">Source Token</label>
                      <select
                        value={dcaConfig.sourceToken}
                        onChange={(e) => setDcaConfig({ ...dcaConfig, sourceToken: e.target.value })}
                        className="w-full px-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-[#3B00B9] focus:border-transparent bg-white text-gray-900"
                      >
                        <option value="USDC">USDC</option>
                        <option value="USDT">USDT</option>
                        <option value="DAI">DAI</option>
                      </select>
                    </div>

                    <div>
                      <label className="block text-sm font-semibold text-gray-900 mb-2">Target Token</label>
                      <select
                        value={dcaConfig.targetToken}
                        onChange={(e) => setDcaConfig({ ...dcaConfig, targetToken: e.target.value })}
                        className="w-full px-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-[#3B00B9] focus:border-transparent bg-white text-gray-900"
                      >
                        <option value="ETH">ETH</option>
                        <option value="WBTC">WBTC</option>
                        <option value="LINK">LINK</option>
                      </select>
                    </div>

                    <div className="md:col-span-2">
                      <label className="block text-sm font-semibold text-gray-900 mb-2">Amount Per Purchase</label>
                      <input
                        type="text"
                        value={dcaConfig.amountPerPurchase}
                        onChange={(e) => setDcaConfig({ ...dcaConfig, amountPerPurchase: e.target.value })}
                        placeholder="1000000"
                        className="w-full px-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-[#3B00B9] focus:border-transparent font-mono bg-white text-gray-900"
                      />
                      <p className="text-xs text-gray-500 mt-2">Enter amount in smallest unit (e.g., 1000000 = 1 USDC with 6 decimals)</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-3 p-4 bg-[#3B00B9]/5 border border-[#3B00B9]/20 rounded-lg">
                    <input
                      type="checkbox"
                      id="dca-enabled"
                      checked={dcaConfig.enabled}
                      onChange={(e) => setDcaConfig({ ...dcaConfig, enabled: e.target.checked })}
                      className="w-5 h-5 text-[#3B00B9] border-gray-300 rounded focus:ring-[#3B00B9]"
                    />
                    <label htmlFor="dca-enabled" className="text-sm font-semibold text-gray-900">
                      Enable automated DCA execution
                    </label>
                  </div>

                  <div className="flex gap-4 pt-4">
                    <button
                      type="submit"
                      className="flex-1 bg-[#3B00B9] text-white font-semibold py-3 px-6 rounded-lg hover:bg-[#1B025A] transition-colors"
                    >
                      Save Configuration
                    </button>
                    <button
                      type="button"
                      onClick={handleExecuteNow}
                      className="bg-[#18C39F] text-white font-semibold py-3 px-6 rounded-lg hover:bg-[#10B981] transition-colors"
                    >
                      Execute Now
                    </button>
                  </div>
                </form>
              ) : (
                <form onSubmit={handleRebalanceSubmit} className="space-y-6">
                  <div>
                    <label className="block text-sm font-semibold text-gray-900 mb-2">Chain</label>
                    <select
                      value={rebalanceConfig.chain}
                      onChange={(e) => setRebalanceConfig({ ...rebalanceConfig, chain: e.target.value })}
                      className="w-full px-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-[#3B00B9] focus:border-transparent bg-white text-gray-900"
                    >
                      <option value="Sepolia">Sepolia Testnet</option>
                      <option value="Ethereum">Ethereum Mainnet</option>
                    </select>
                  </div>

                  <div>
                    <label className="block text-sm font-semibold text-gray-900 mb-4">Portfolio Allocation</label>
                    <div className="space-y-4">
                      {rebalanceConfig.portfolio.map((asset, index) => (
                        <div key={asset.token} className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
                          <div className="flex items-center justify-between mb-3">
                            <span className="font-bold text-gray-900">{asset.token}</span>
                            <div className="flex gap-4 text-sm">
                              <span className="text-gray-600">Target: <span className="font-semibold text-[#3B00B9]">{asset.targetPercent}%</span></span>
                              <span className="text-gray-600">Current: <span className="font-semibold text-gray-900">{asset.currentPercent}%</span></span>
                            </div>
                          </div>
                          <input
                            type="range"
                            min="0"
                            max="100"
                            value={asset.targetPercent}
                            onChange={(e) => {
                              const newPortfolio = [...rebalanceConfig.portfolio];
                              newPortfolio[index].targetPercent = parseInt(e.target.value);
                              setRebalanceConfig({ ...rebalanceConfig, portfolio: newPortfolio });
                            }}
                            className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-[#3B00B9]"
                          />
                        </div>
                      ))}
                    </div>
                    <p className="text-xs text-gray-500 mt-2">
                      Total: {rebalanceConfig.portfolio.reduce((sum, a) => sum + a.targetPercent, 0)}%
                    </p>
                  </div>

                  <div>
                    <label className="block text-sm font-semibold text-gray-900 mb-2">Rebalance Threshold (%)</label>
                    <input
                      type="number"
                      value={rebalanceConfig.rebalanceThreshold}
                      onChange={(e) => setRebalanceConfig({ ...rebalanceConfig, rebalanceThreshold: parseInt(e.target.value) })}
                      min="1"
                      max="50"
                      className="w-full px-4 py-3 border border-gray-200 rounded-lg focus:ring-2 focus:ring-[#3B00B9] focus:border-transparent bg-white text-gray-900"
                    />
                    <p className="text-xs text-gray-500 mt-2">Trigger rebalancing when any asset deviates by this percentage</p>
                  </div>

                  <div className="flex items-center gap-3 p-4 bg-[#3B00B9]/5 border border-[#3B00B9]/20 rounded-lg">
                    <input
                      type="checkbox"
                      id="rebalance-enabled"
                      checked={rebalanceConfig.enabled}
                      onChange={(e) => setRebalanceConfig({ ...rebalanceConfig, enabled: e.target.checked })}
                      className="w-5 h-5 text-[#3B00B9] border-gray-300 rounded focus:ring-[#3B00B9]"
                    />
                    <label htmlFor="rebalance-enabled" className="text-sm font-semibold text-gray-900">
                      Enable automated rebalancing
                    </label>
                  </div>

                  <div className="flex gap-4 pt-4">
                    <button
                      type="submit"
                      className="flex-1 bg-[#3B00B9] text-white font-semibold py-3 px-6 rounded-lg hover:bg-[#1B025A] transition-colors"
                    >
                      Save Configuration
                    </button>
                    <button
                      type="button"
                      onClick={handleExecuteNow}
                      className="bg-[#18C39F] text-white font-semibold py-3 px-6 rounded-lg hover:bg-[#10B981] transition-colors"
                    >
                      Analyze & Rebalance
                    </button>
                  </div>
                </form>
              )}
            </div>
          </div>

          {/* Status Sidebar */}
          <div className="space-y-6">
            <div>
              <h3 className="text-xl font-bold text-[#1B025A] mb-4">Strategy Status</h3>
              <div className="border border-gray-200 rounded-xl p-6 bg-white">
                <div className="space-y-4">
                  <div className="flex items-center justify-between pb-3 border-b border-gray-200">
                    <span className="text-sm font-semibold text-gray-700">Status</span>
                    <div className="flex items-center gap-2">
                      <div className="h-2 w-2 bg-[#18C39F] rounded-full animate-pulse"></div>
                      <span className="text-sm font-bold text-[#18C39F]">Active</span>
                    </div>
                  </div>
                  <div className="py-2">
                    <div className="text-xs text-gray-500 mb-1">Next Execution</div>
                    <div className="text-sm font-semibold text-gray-900">2025-12-12 10:30 UTC</div>
                  </div>
                  <div className="py-2">
                    <div className="text-xs text-gray-500 mb-1">Total Executions</div>
                    <div className="text-sm font-semibold text-gray-900">127 successful</div>
                  </div>
                  <div className="py-2">
                    <div className="text-xs text-gray-500 mb-1">Success Rate</div>
                    <div className="text-sm font-semibold text-gray-900">98.4%</div>
                  </div>
                </div>
              </div>
            </div>

            <div>
              <h3 className="text-xl font-bold text-[#1B025A] mb-4">Performance</h3>
              <div className="border border-gray-200 rounded-xl p-6 bg-gradient-to-br from-[#18C39F]/10 to-[#10B981]/10">
                <div className="text-4xl font-bold text-[#18C39F] mb-1">+24.3%</div>
                <div className="text-sm text-gray-600 font-medium">Last 30 days</div>
              </div>
            </div>
          </div>
        </div>

        {/* Execution History */}
        <div>
          <h2 className="text-3xl font-bold text-[#1B025A] mb-6">Execution History</h2>
          <div className="border border-gray-200 rounded-xl overflow-hidden bg-white">
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-gray-50 border-b border-gray-200">
                  <tr>
                    <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Timestamp</th>
                    <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Strategy</th>
                    <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Action</th>
                    <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Status</th>
                    <th className="text-left py-4 px-6 text-sm font-bold text-gray-900">Transaction</th>
                  </tr>
                </thead>
                <tbody>
                  {executionHistory.map((entry) => (
                    <tr key={entry.id} className="border-b border-gray-100 hover:bg-gray-50 transition-colors">
                      <td className="py-4 px-6 text-sm text-gray-900 font-mono">{entry.timestamp}</td>
                      <td className="py-4 px-6">
                        <span className={`inline-block px-3 py-1 rounded-full text-xs font-semibold ${
                          entry.strategy === 'DCA'
                            ? 'bg-[#3B00B9]/10 text-[#3B00B9]'
                            : 'bg-[#ED1E79]/10 text-[#ED1E79]'
                        }`}>
                          {entry.strategy}
                        </span>
                      </td>
                      <td className="py-4 px-6 text-sm text-gray-900">{entry.action}</td>
                      <td className="py-4 px-6">
                        <span className={`inline-block px-3 py-1 rounded-full text-xs font-semibold ${
                          entry.status === 'Success'
                            ? 'bg-[#18C39F]/10 text-[#18C39F]'
                            : entry.status === 'Pending Signatures'
                            ? 'bg-[#F59E0B]/10 text-[#F59E0B]'
                            : 'bg-[#EF4444]/10 text-[#EF4444]'
                        }`}>
                          {entry.status}
                        </span>
                      </td>
                      <td className="py-4 px-6">
                        {entry.txHash ? (
                          <a
                            href={`https://sepolia.etherscan.io/tx/${entry.txHash}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-[#3B00B9] hover:text-[#1B025A] font-mono text-sm hover:underline"
                          >
                            {entry.txHash}
                          </a>
                        ) : (
                          <span className="text-gray-400 text-sm">Pending</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
