/**
 * Bitcoin Dollar Cost Averaging (DCA) Strategy
 *
 * Automated Bitcoin accumulation strategy that purchases a fixed amount
 * of Bitcoin at regular intervals, regardless of price.
 *
 * Benefits:
 * - Reduces impact of volatility
 * - Removes emotional decision-making
 * - Builds Bitcoin position over time
 */

import { ChainGuardClient, ActionResult } from '@chainguarsdk/sdk';

export interface BitcoinDCAConfig {
  // Amount to purchase each interval (in satoshis)
  amountPerPurchase: bigint;

  // Purchase interval in seconds (e.g., 86400 = daily, 604800 = weekly)
  intervalSeconds: number;

  // Bitcoin network: "Bitcoin" or "BitcoinTestnet"
  network: string;

  // Recipient Bitcoin address (where BTC will be sent)
  recipientAddress: string;

  // Maximum total amount to accumulate (in satoshis, optional)
  maxTotalAmount?: bigint;
}

export class BitcoinDCAStrategy {
  private client: ChainGuardClient;
  private config: BitcoinDCAConfig;
  private lastExecutionTime: number = 0;
  private totalAccumulated: bigint = BigInt(0);

  constructor(client: ChainGuardClient, config: BitcoinDCAConfig) {
    this.client = client;
    this.config = config;
  }

  /**
   * Check if it's time to execute the next purchase
   */
  shouldExecute(): boolean {
    const now = Date.now() / 1000; // Convert to seconds
    const timeSinceLastExecution = now - this.lastExecutionTime;

    // Check if interval has passed
    if (timeSinceLastExecution < this.config.intervalSeconds) {
      return false;
    }

    // Check if we've reached max total amount
    if (
      this.config.maxTotalAmount &&
      this.totalAccumulated >= this.config.maxTotalAmount
    ) {
      console.log('⚠️ Maximum total amount reached. Stopping DCA strategy.');
      return false;
    }

    return true;
  }

  /**
   * Execute Bitcoin DCA purchase
   */
  async execute(): Promise<ActionResult> {
    if (!this.shouldExecute()) {
      throw new Error('Not time to execute yet or max amount reached');
    }

    console.log('\n🪙 Executing Bitcoin DCA Strategy');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log(`Network: ${this.config.network}`);
    console.log(`Amount: ${this.config.amountPerPurchase} satoshis`);
    console.log(`Recipient: ${this.config.recipientAddress}`);

    try {
      // Get canister's Bitcoin address
      const addressResult = await this.client.getBitcoinAddress(
        this.config.network
      );

      if ('Err' in addressResult) {
        throw new Error(`Failed to get Bitcoin address: ${addressResult.Err}`);
      }

      const canisterAddress = addressResult.Ok;
      console.log(`\nCanister Bitcoin Address: ${canisterAddress}`);

      // Check if we need to transfer from canister to recipient
      // In a real scenario, the canister would:
      // 1. Receive Bitcoin from an exchange or external source
      // 2. Accumulate in the canister's address
      // 3. Transfer to recipient address periodically

      // Execute Bitcoin transfer
      console.log('\n📤 Sending Bitcoin transfer request...');
      const result = await this.client.bitcoinTransfer(
        this.config.recipientAddress,
        this.config.amountPerPurchase,
        this.config.network
      );

      // Handle result
      if ('Executed' in result) {
        const execution = result.Executed;
        if (execution.success) {
          console.log('✅ Bitcoin transfer executed successfully!');
          console.log(`TX Hash: ${execution.tx_hash[0] || 'N/A'}`);

          // Update tracking
          this.lastExecutionTime = Date.now() / 1000;
          this.totalAccumulated += this.config.amountPerPurchase;

          console.log(`\n📊 Total Accumulated: ${this.totalAccumulated} satoshis`);
          if (this.config.maxTotalAmount) {
            const remaining = this.config.maxTotalAmount - this.totalAccumulated;
            console.log(`   Remaining: ${remaining} satoshis`);
          }
        } else {
          console.error('❌ Bitcoin transfer failed:', execution.error[0]);
        }
      } else if ('PendingSignatures' in result) {
        console.log('⏳ Transfer requires threshold signatures');
        console.log(`   Request ID: ${result.PendingSignatures.id}`);
        console.log(
          `   Required: ${result.PendingSignatures.required_signatures} signatures`
        );
      } else if ('Denied' in result) {
        console.error('❌ Transfer denied by policy:', result.Denied.reason);
      }

      return result;
    } catch (error) {
      console.error('💥 Bitcoin DCA execution error:', error);
      throw error;
    }
  }

  /**
   * Get strategy statistics
   */
  getStats() {
    return {
      totalAccumulated: this.totalAccumulated,
      lastExecutionTime: this.lastExecutionTime,
      nextExecutionTime: this.lastExecutionTime + this.config.intervalSeconds,
      maxTotalAmount: this.config.maxTotalAmount,
      amountPerPurchase: this.config.amountPerPurchase,
      network: this.config.network,
    };
  }

  /**
   * Estimate total purchases remaining
   */
  estimateRemainingPurchases(): number {
    if (!this.config.maxTotalAmount) {
      return Infinity;
    }

    const remaining = this.config.maxTotalAmount - this.totalAccumulated;
    return Number(remaining / this.config.amountPerPurchase);
  }

  /**
   * Reset accumulated total (for testing or restarting strategy)
   */
  reset() {
    this.totalAccumulated = BigInt(0);
    this.lastExecutionTime = 0;
    console.log('🔄 Bitcoin DCA strategy reset');
  }
}

/**
 * Example usage:
 *
 * const config = {
 *   amountPerPurchase: BigInt(100000), // 0.001 BTC = 100,000 satoshis
 *   intervalSeconds: 604800,           // Weekly
 *   network: "BitcoinTestnet",
 *   recipientAddress: "tb1q...",       // Your Bitcoin testnet address
 *   maxTotalAmount: BigInt(10000000),  // 0.1 BTC total
 * };
 *
 * const strategy = new BitcoinDCAStrategy(client, config);
 *
 * // Check if ready to execute
 * if (strategy.shouldExecute()) {
 *   await strategy.execute();
 * }
 */
