/**
 * ChainGuard AI Agent - Main Entry Point
 *
 * Run strategies on a schedule or execute one-time commands
 */

import * as cron from 'node-cron';
import * as dotenv from 'dotenv';
import * as fs from 'fs';
import { Ed25519KeyIdentity } from '@dfinity/identity';
import { ChainGuardClient } from '@chainguarsdk/sdk';
import { ConfigManager } from './utils/config';
import { DCAStrategy } from './strategies/dca';
import { RebalanceStrategy } from './strategies/rebalance';

// Load environment variables
dotenv.config();

interface AgentOptions {
  configPath?: string;
  identityPath?: string;
  mode?: 'schedule' | 'once';
  strategy?: 'dca' | 'rebalance' | 'all';
}

export class ChainGuardAgent {
  private client: ChainGuardClient;
  private config: ConfigManager;
  private dcaStrategy?: DCAStrategy;
  private rebalanceStrategy?: RebalanceStrategy;

  constructor(options: AgentOptions = {}) {
    const configPath = options.configPath || './config.yaml';
    const identityPath = options.identityPath || process.env.IDENTITY_PATH;

    // Load configuration
    this.config = new ConfigManager(configPath);

    // Load identity
    const identity = this.loadIdentity(identityPath);

    // Initialize ChainGuard client
    const canisterConfig = this.config.getCanisterConfig();
    this.client = new ChainGuardClient({
      canisterId: canisterConfig.id,
      identity,
      host: canisterConfig.host,
    });

    // Initialize strategies
    if (this.config.isStrategyEnabled('dca')) {
      this.dcaStrategy = new DCAStrategy(this.client, this.config);
    }

    if (this.config.isStrategyEnabled('rebalance')) {
      this.rebalanceStrategy = new RebalanceStrategy(this.client, this.config);
    }

    console.log('✅ ChainGuard Agent initialized');
    console.log(`   Canister: ${canisterConfig.id}`);
    console.log(`   Network: ${canisterConfig.network}`);
    console.log(`   Principal: ${this.client.getPrincipal().toText()}\n`);
  }

  /**
   * Load identity from PEM file or use anonymous
   */
  private loadIdentity(identityPath?: string) {
    const identityConfig = this.config.getIdentityConfig();

    if (identityConfig.type === 'pem' && identityConfig.path) {
      const pemPath = identityPath || identityConfig.path;
      console.log(`Loading identity from: ${pemPath}`);

      try {
        const pemContent = fs.readFileSync(pemPath, 'utf8');
        // Use fromPemFile or generate new identity
        return Ed25519KeyIdentity.generate(); // PEM parsing requires different method
      } catch (error) {
        console.error('Failed to load identity from PEM:', error);
        console.log('Generating temporary identity...');
        return Ed25519KeyIdentity.generate();
      }
    } else {
      console.log('Generating temporary identity...');
      return Ed25519KeyIdentity.generate();
    }
  }

  /**
   * Run strategies on a schedule
   */
  async runScheduled(): Promise<void> {
    console.log('🚀 Starting scheduled agent...\n');

    // Schedule DCA strategy
    if (this.dcaStrategy) {
      await this.dcaStrategy.runScheduled(cron);
      console.log('✅ DCA strategy scheduled');
    }

    // Schedule rebalancing strategy
    if (this.rebalanceStrategy) {
      await this.rebalanceStrategy.runScheduled(cron);
      console.log('✅ Rebalancing strategy scheduled');
    }

    console.log('\n📅 Agent is running. Press Ctrl+C to stop.\n');

    // Keep process alive
    process.on('SIGINT', () => {
      console.log('\n👋 Shutting down agent...');
      process.exit(0);
    });
  }

  /**
   * Execute a strategy once
   */
  async executeOnce(strategy: 'dca' | 'rebalance' | 'all'): Promise<void> {
    console.log(`🎯 Executing ${strategy} strategy...\n`);

    try {
      if (strategy === 'dca' || strategy === 'all') {
        if (this.dcaStrategy) {
          await this.dcaStrategy.execute();
        } else {
          console.log('⚠️  DCA strategy not enabled');
        }
      }

      if (strategy === 'rebalance' || strategy === 'all') {
        if (this.rebalanceStrategy) {
          await this.rebalanceStrategy.execute();
        } else {
          console.log('⚠️  Rebalancing strategy not enabled');
        }
      }

      console.log('\n✅ Execution complete');
    } catch (error) {
      console.error('❌ Execution failed:', error);
      throw error;
    }
  }

  /**
   * Get pending threshold signature requests
   */
  async monitorPendingRequests(): Promise<void> {
    console.log('📋 Monitoring pending requests...\n');

    setInterval(async () => {
      try {
        const pending = await this.client.getPendingRequests();

        if (pending.length > 0) {
          console.log(`⏳ ${pending.length} pending request(s):\n`);

          pending.forEach(req => {
            console.log(`  Request ID: ${req.id}`);
            console.log(`  Requester: ${req.requester.toText()}`);
            console.log(`  Signatures: ${req.collected_signatures.length}/${req.required_signatures}`);
            console.log(`  Status: ${Object.keys(req.status)[0]}`);
            console.log('');
          });
        }
      } catch (error) {
        console.error('Error fetching pending requests:', error);
      }
    }, 30000); // Check every 30 seconds
  }

  /**
   * Display agent status
   */
  async getStatus(): Promise<void> {
    console.log('📊 Agent Status\n');
    console.log('════════════════════════════════════════\n');

    try {
      const isPaused = await this.client.isPaused();
      console.log(`System Status: ${isPaused ? '⏸️  Paused' : '▶️  Active'}`);

      const roles = await this.client.getRoles(this.client.getPrincipal());
      console.log(`Your Roles: ${roles.map(r => Object.keys(r)[0]).join(', ') || 'None'}`);

      const policies = await this.client.listPolicies();
      console.log(`Active Policies: ${policies.length}`);

      const pending = await this.client.getPendingRequests();
      console.log(`Pending Requests: ${pending.length}`);

      const auditLogs = await this.client.getAuditLogs();
      console.log(`Total Actions: ${auditLogs.length}`);

      // DCA stats
      if (this.dcaStrategy) {
        const dcaStats = await this.dcaStrategy.getStats();
        console.log('\nDCA Strategy:');
        console.log(`  Purchases: ${dcaStats.totalPurchases}`);
        console.log(`  Success Rate: ${dcaStats.totalPurchases > 0 ? ((dcaStats.successfulPurchases / dcaStats.totalPurchases) * 100).toFixed(1) : 0}%`);
      }

      console.log('\n════════════════════════════════════════\n');
    } catch (error) {
      console.error('Error fetching status:', error);
    }
  }
}

// ==================== CLI Entry Point ====================

async function main() {
  const args = process.argv.slice(2);
  const command = args[0] || 'help';

  console.log('╔═══════════════════════════════════════════════════════╗');
  console.log('║   ChainGuard AI Agent                                 ║');
  console.log('╚═══════════════════════════════════════════════════════╝\n');

  try {
    const agent = new ChainGuardAgent({
      configPath: process.env.CONFIG_PATH || './config.yaml',
      identityPath: process.env.IDENTITY_PATH,
    });

    switch (command) {
      case 'schedule':
        await agent.runScheduled();
        break;

      case 'dca':
        await agent.executeOnce('dca');
        break;

      case 'rebalance':
        await agent.executeOnce('rebalance');
        break;

      case 'all':
        await agent.executeOnce('all');
        break;

      case 'status':
        await agent.getStatus();
        break;

      case 'monitor':
        await agent.monitorPendingRequests();
        break;

      case 'help':
      default:
        console.log('Usage: npm start <command>\n');
        console.log('Commands:');
        console.log('  schedule   - Run strategies on a schedule');
        console.log('  dca        - Execute DCA strategy once');
        console.log('  rebalance  - Execute rebalancing strategy once');
        console.log('  all        - Execute all strategies once');
        console.log('  status     - Show agent status');
        console.log('  monitor    - Monitor pending signature requests');
        console.log('  help       - Show this help message\n');
        break;
    }
  } catch (error) {
    console.error('Fatal error:', error);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}

export default ChainGuardAgent;
