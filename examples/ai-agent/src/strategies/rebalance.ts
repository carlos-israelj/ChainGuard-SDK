/**
 * Portfolio Rebalancing Strategy
 *
 * Automatically rebalances a portfolio to maintain target allocations.
 * Triggers when any asset deviates from target by more than threshold.
 */

import { ChainGuardClient } from '../utils/chainguard-client';
import { ConfigManager } from '../utils/config';
import { ActionResult } from '../types/chainguard';

interface PortfolioAsset {
  token: string;
  address: string;
  targetPercentage: number;
  currentBalance: bigint;
  currentPercentage: number;
  deviation: number;
}

interface RebalanceAction {
  from: string;
  to: string;
  amount: bigint;
}

export class RebalanceStrategy {
  private client: ChainGuardClient;
  private config: ConfigManager;

  constructor(client: ChainGuardClient, config: ConfigManager) {
    this.client = client;
    this.config = config;
  }

  /**
   * Analyze portfolio and determine if rebalancing is needed
   */
  async analyzePortfolio(): Promise<{
    needsRebalance: boolean;
    assets: PortfolioAsset[];
    actions: RebalanceAction[];
  }> {
    const strategyConfig = this.config.getStrategyConfig('rebalance');

    if (!strategyConfig?.enabled) {
      throw new Error('Rebalance strategy is not enabled');
    }

    const params = strategyConfig.params;
    const chain = params.chain;

    console.log('\n========================================');
    console.log('📊 Portfolio Analysis');
    console.log('========================================');

    // Mock balances - in production, fetch from on-chain sources
    const balances = await this.getPortfolioBalances(chain, params.portfolio);

    // Calculate total value
    const totalValue = balances.reduce((sum, asset) => sum + asset.currentBalance, BigInt(0));

    // Calculate current percentages and deviations
    const assets: PortfolioAsset[] = balances.map(balance => {
      const currentPercentage = Number(balance.currentBalance * BigInt(10000) / totalValue) / 100;
      const targetPercentage = params.portfolio.find(p => p.token === balance.token)?.targetPercentage || 0;
      const deviation = Math.abs(currentPercentage - targetPercentage);

      return {
        ...balance,
        currentPercentage,
        targetPercentage,
        deviation,
      };
    });

    // Check if any asset exceeds rebalance threshold
    const maxDeviation = Math.max(...assets.map(a => a.deviation));
    const needsRebalance = maxDeviation > params.rebalanceThreshold;

    console.log('Portfolio Status:');
    assets.forEach(asset => {
      console.log(
        `  ${asset.token}: ${asset.currentPercentage.toFixed(2)}% (target: ${asset.targetPercentage}%, deviation: ${asset.deviation.toFixed(2)}%)`
      );
    });
    console.log(`Max Deviation: ${maxDeviation.toFixed(2)}%`);
    console.log(`Threshold: ${params.rebalanceThreshold}%`);
    console.log(`Needs Rebalance: ${needsRebalance ? 'YES ✅' : 'NO ❌'}`);
    console.log('========================================\n');

    // Calculate rebalance actions
    const actions = needsRebalance ? this.calculateRebalanceActions(assets, totalValue) : [];

    return {
      needsRebalance,
      assets,
      actions,
    };
  }

  /**
   * Execute portfolio rebalancing
   */
  async execute(): Promise<ActionResult[]> {
    const analysis = await this.analyzePortfolio();

    if (!analysis.needsRebalance) {
      console.log('[Rebalance] Portfolio is balanced. No action needed.');
      return [];
    }

    console.log('\n========================================');
    console.log('⚖️ Executing Rebalance');
    console.log('========================================');

    const results: ActionResult[] = [];
    const strategyConfig = this.config.getStrategyConfig('rebalance');
    const chain = strategyConfig!.params.chain;

    for (const action of analysis.actions) {
      console.log(`\nSwapping ${action.amount} ${action.from} → ${action.to}`);

      try {
        const result = await this.client.swap(
          chain,
          action.from,
          action.to,
          action.amount,
          BigInt(1), // Minimum slippage protection
          3000 // 0.3% fee tier
        );

        results.push(result);
        this.logResult(result);

        // Wait between transactions to avoid nonce conflicts
        await this.sleep(5000);
      } catch (error) {
        console.error(`Failed to execute swap ${action.from} → ${action.to}:`, error);
      }
    }

    console.log('========================================\n');
    return results;
  }

  /**
   * Run rebalancing strategy on a schedule
   */
  async runScheduled(cron: any): Promise<void> {
    const strategyConfig = this.config.getStrategyConfig('rebalance');

    if (!strategyConfig?.enabled) {
      console.log('[Rebalance] Strategy is disabled');
      return;
    }

    const interval = strategyConfig.interval || '0 0 * * 0'; // Weekly on Sunday by default

    console.log(`[Rebalance] Scheduling strategy with interval: ${interval}`);

    cron.schedule(interval, async () => {
      try {
        await this.execute();
      } catch (error) {
        console.error('[Rebalance] Execution failed:', error);
      }
    });
  }

  // ==================== Helper Methods ====================

  /**
   * Get current portfolio balances
   * In production, this would fetch from on-chain sources or price oracles
   */
  private async getPortfolioBalances(
    chain: string,
    portfolio: Array<{ token: string; targetPercentage: number }>
  ): Promise<Array<{ token: string; address: string; currentBalance: bigint }>> {
    // Mock implementation - replace with real balance fetching
    return portfolio.map(asset => ({
      token: asset.token,
      address: this.getTokenAddress(chain, asset.token),
      currentBalance: BigInt(Math.floor(Math.random() * 1000000000000000000)), // Random for demo
    }));
  }

  /**
   * Calculate which swaps are needed to rebalance portfolio
   */
  private calculateRebalanceActions(
    assets: PortfolioAsset[],
    totalValue: bigint
  ): RebalanceAction[] {
    const actions: RebalanceAction[] = [];

    // Find assets that need to sell (above target)
    const toSell = assets.filter(a => a.currentPercentage > a.targetPercentage);
    // Find assets that need to buy (below target)
    const toBuy = assets.filter(a => a.currentPercentage < a.targetPercentage);

    // Simple greedy matching - in production, use more sophisticated algorithm
    for (const sell of toSell) {
      const excessPercentage = sell.currentPercentage - sell.targetPercentage;
      const excessAmount = (totalValue * BigInt(Math.floor(excessPercentage * 100))) / BigInt(10000);

      for (const buy of toBuy) {
        if (excessAmount > BigInt(0)) {
          const deficitPercentage = buy.targetPercentage - buy.currentPercentage;
          const deficitAmount = (totalValue * BigInt(Math.floor(deficitPercentage * 100))) / BigInt(10000);

          const swapAmount = excessAmount < deficitAmount ? excessAmount : deficitAmount;

          if (swapAmount > BigInt(0)) {
            actions.push({
              from: sell.address,
              to: buy.address,
              amount: swapAmount,
            });
          }
        }
      }
    }

    return actions;
  }

  private getTokenAddress(chain: string, symbol: string): string {
    if (symbol === 'ETH' || symbol === 'WETH') {
      return symbol;
    }

    const address = this.config.getTokenAddress(chain, symbol);
    if (!address) {
      throw new Error(`Token ${symbol} not found in chain ${chain} config`);
    }

    return address;
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private logResult(result: ActionResult): void {
    if ('Executed' in result) {
      const exec = result.Executed;
      if (exec.success && exec.tx_hash.length > 0) {
        console.log(`✅ Swap executed: ${exec.tx_hash[0]}`);
      } else {
        console.log('❌ Swap failed');
        if (exec.error && exec.error.length > 0) {
          console.log(`Error: ${exec.error[0]}`);
        }
      }
    } else if ('PendingSignatures' in result) {
      const pending = result.PendingSignatures;
      console.log(`⏳ Swap pending approval (Request ID: ${pending.id})`);
    } else if ('Denied' in result) {
      console.log(`🚫 Swap denied: ${result.Denied.reason}`);
    }
  }
}
