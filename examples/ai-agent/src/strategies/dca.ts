/**
 * Dollar Cost Averaging (DCA) Strategy
 *
 * Automatically purchases a fixed amount of a target token at regular intervals,
 * regardless of price. This reduces the impact of volatility.
 */

import { ChainGuardClient } from '@chainguarsdk/sdk';
import { ConfigManager } from '../utils/config';
import { ActionResult } from '@chainguarsdk/sdk';

export class DCAStrategy {
  private client: ChainGuardClient;
  private config: ConfigManager;

  constructor(client: ChainGuardClient, config: ConfigManager) {
    this.client = client;
    this.config = config;
  }

  /**
   * Execute a single DCA purchase
   */
  async execute(): Promise<ActionResult> {
    const strategyConfig = this.config.getStrategyConfig('dca');

    if (!strategyConfig?.enabled) {
      throw new Error('DCA strategy is not enabled');
    }

    const params = strategyConfig.params;
    console.log('\n========================================');
    console.log('🔄 DCA Strategy Execution');
    console.log('========================================');
    console.log(`Chain: ${params.chain}`);
    console.log(`Buying: ${params.targetToken}`);
    console.log(`Using: ${params.sourceToken}`);
    console.log(`Amount: ${params.amountPerPurchase}`);
    console.log('========================================\n');

    // Get token addresses
    const sourceTokenAddress = this.getTokenAddress(params.chain, params.sourceToken);
    const targetTokenAddress = this.getTokenAddress(params.chain, params.targetToken);

    // Convert amount to wei/smallest unit
    const amountIn = this.parseAmount(params.amountPerPurchase);
    const minAmountOut = params.minAmountOut
      ? this.parseAmount(params.minAmountOut)
      : BigInt(1); // Minimum protection

    console.log(`[DCA] Executing swap: ${amountIn} ${params.sourceToken} → ${params.targetToken}`);

    // Execute swap through ChainGuard
    const result = await this.client.swap(
      params.chain,
      sourceTokenAddress,
      targetTokenAddress,
      amountIn,
      minAmountOut,
      3000 // 0.3% fee tier for Uniswap V3
    );

    this.logResult(result);
    return result;
  }

  /**
   * Run DCA strategy on a schedule
   */
  async runScheduled(cron: any): Promise<void> {
    const strategyConfig = this.config.getStrategyConfig('dca');

    if (!strategyConfig?.enabled) {
      console.log('[DCA] Strategy is disabled');
      return;
    }

    const interval = strategyConfig.interval || '0 0 * * *'; // Daily at midnight by default

    console.log(`[DCA] Scheduling strategy with interval: ${interval}`);

    cron.schedule(interval, async () => {
      try {
        await this.execute();
      } catch (error) {
        console.error('[DCA] Execution failed:', error);
      }
    });
  }

  /**
   * Get historical DCA executions from audit logs
   */
  async getHistory(limit: number = 10): Promise<any[]> {
    const logs = await this.client.getAuditLogs();

    return logs
      .filter(log => log.action_type === 'swap')
      .slice(-limit)
      .map(log => ({
        timestamp: log.timestamp,
        params: JSON.parse(log.action_params),
        result: log.execution_result,
        success: log.execution_result?.[0]?.success ?? false,
      }));
  }

  /**
   * Calculate DCA statistics
   */
  async getStats(): Promise<{
    totalPurchases: number;
    successfulPurchases: number;
    failedPurchases: number;
    totalSpent: bigint;
    averagePrice?: number;
  }> {
    const history = await this.getHistory(1000);

    const totalPurchases = history.length;
    const successfulPurchases = history.filter(h => h.success).length;
    const failedPurchases = totalPurchases - successfulPurchases;

    const totalSpent = history
      .filter(h => h.success)
      .reduce((sum, h) => sum + BigInt(h.params.amount_in || 0), BigInt(0));

    return {
      totalPurchases,
      successfulPurchases,
      failedPurchases,
      totalSpent,
    };
  }

  // ==================== Helper Methods ====================

  private getTokenAddress(chain: string, symbol: string): string {
    // Special case for native tokens
    if (symbol === 'ETH' || symbol === 'WETH') {
      return symbol;
    }

    const address = this.config.getTokenAddress(chain, symbol);
    if (!address) {
      throw new Error(`Token ${symbol} not found in chain ${chain} config`);
    }

    return address;
  }

  private parseAmount(amount: string): bigint {
    // Simple parser - assumes wei/smallest unit
    // In production, should handle decimals properly
    return BigInt(amount);
  }

  private logResult(result: ActionResult): void {
    console.log('\n========================================');

    if ('Executed' in result) {
      const exec = result.Executed;
      if (exec.success && exec.tx_hash.length > 0) {
        console.log('✅ DCA Execution Successful!');
        console.log(`Transaction: ${exec.tx_hash[0]}`);
        console.log(`Chain: ${exec.chain}`);
      } else {
        console.log('❌ DCA Execution Failed');
        if (exec.error && exec.error.length > 0) {
          console.log(`Error: ${exec.error[0]}`);
        }
      }
    } else if ('PendingSignatures' in result) {
      const pending = result.PendingSignatures;
      console.log('⏳ DCA Execution Pending Approval');
      console.log(`Request ID: ${pending.id}`);
      console.log(`Signatures: ${pending.collected_signatures.length}/${pending.required_signatures}`);
      console.log(`Expires at: ${new Date(Number(pending.expires_at) / 1000000).toISOString()}`);
    } else if ('Denied' in result) {
      console.log('🚫 DCA Execution Denied');
      console.log(`Reason: ${result.Denied.reason}`);
    }

    console.log('========================================\n');
  }
}

// ==================== Standalone Execution ====================

/**
 * Run DCA strategy as a standalone script
 */
export async function runDCA() {
  console.log('Starting DCA Strategy...\n');

  // This would be implemented in the main index.ts
  // Placeholder for demonstration
  console.log('Please run via: npm run dca');
}

if (require.main === module) {
  runDCA().catch(console.error);
}
