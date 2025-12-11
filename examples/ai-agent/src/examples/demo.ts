/**
 * Comprehensive Demo - ChainGuard AI Agent
 *
 * Demonstrates:
 * 1. Policy-based action execution
 * 2. Threshold signature workflow
 * 3. DCA strategy
 * 4. Portfolio rebalancing
 * 5. Audit log tracking
 */

import { Ed25519KeyIdentity } from '@dfinity/identity';
import { ChainGuardClient } from '../utils/chainguard-client';
import { ConfigManager } from '../utils/config';
import { DCAStrategy } from '../strategies/dca';
import { RebalanceStrategy } from '../strategies/rebalance';

async function main() {
  console.log('\n╔═══════════════════════════════════════════════════════╗');
  console.log('║   ChainGuard SDK - AI Agent Demo                     ║');
  console.log('║   Secure Multi-Chain Transactions with ICP           ║');
  console.log('╚═══════════════════════════════════════════════════════╝\n');

  // ==================== Setup ====================

  console.log('📋 Step 1: Loading Configuration\n');

  const config = new ConfigManager('./config.yaml');
  const canisterConfig = config.getCanisterConfig();

  console.log(`Canister ID: ${canisterConfig.id}`);
  console.log(`Network: ${canisterConfig.network}\n`);

  // Create identity (in production, load from PEM file)
  const identity = Ed25519KeyIdentity.generate();
  console.log(`Agent Identity: ${identity.getPrincipal().toText()}\n`);

  // Initialize ChainGuard client
  const client = new ChainGuardClient({
    canisterId: canisterConfig.id,
    identity,
    host: canisterConfig.host,
  });

  console.log('✅ Client initialized\n');

  // ==================== Demo 1: Simple Transfer (Auto-Approved) ====================

  console.log('═══════════════════════════════════════════════════════');
  console.log('Demo 1: Small Transfer (Auto-Approved by Policy)');
  console.log('═══════════════════════════════════════════════════════\n');

  try {
    const smallTransfer = await client.transfer(
      'Sepolia',
      'ETH',
      '0x648a3e5510f55B4995fA5A22cCD62e2586ACb901',
      BigInt(100000000000000) // 0.0001 ETH
    );

    if ('Executed' in smallTransfer) {
      console.log('✅ Transfer executed immediately (allowed by policy)');
      if (smallTransfer.Executed.tx_hash.length > 0) {
        console.log(`Transaction: ${smallTransfer.Executed.tx_hash[0]}\n`);
      }
    } else if ('Denied' in smallTransfer) {
      console.log(`🚫 Transfer denied: ${smallTransfer.Denied.reason}\n`);
    }
  } catch (error) {
    console.error('Error:', error);
  }

  // ==================== Demo 2: Large Transfer (Requires Threshold) ====================

  console.log('═══════════════════════════════════════════════════════');
  console.log('Demo 2: Large Transfer (Requires Threshold Signatures)');
  console.log('═══════════════════════════════════════════════════════\n');

  try {
    const largeTransfer = await client.transfer(
      'Sepolia',
      'ETH',
      '0x648a3e5510f55B4995fA5A22cCD62e2586ACb901',
      BigInt(5000000000000000000) // 5 ETH - requires approval
    );

    if ('PendingSignatures' in largeTransfer) {
      const pending = largeTransfer.PendingSignatures;
      console.log('⏳ Transfer requires threshold signatures');
      console.log(`Request ID: ${pending.id}`);
      console.log(`Required Signatures: ${pending.required_signatures}`);
      console.log(`Current Signatures: ${pending.collected_signatures.length}`);
      console.log(`Status: Pending Approval`);
      console.log(`Expires: ${new Date(Number(pending.expires_at) / 1000000).toISOString()}\n`);

      // Demonstrate threshold signing workflow
      console.log('--- Threshold Signing Workflow ---\n');

      // In production, different signers would call this
      console.log('Signer 1 approving request...');
      // const signed = await client.signRequest(pending.id);
      // console.log(`✅ Signature collected (${signed.collected_signatures.length}/${signed.required_signatures})\n`);
    } else if ('Denied' in largeTransfer) {
      console.log(`🚫 Transfer denied: ${largeTransfer.Denied.reason}\n`);
    }
  } catch (error) {
    console.error('Error:', error);
  }

  // ==================== Demo 3: DCA Strategy ====================

  console.log('\n═══════════════════════════════════════════════════════');
  console.log('Demo 3: Dollar Cost Averaging (DCA) Strategy');
  console.log('═══════════════════════════════════════════════════════\n');

  if (config.isStrategyEnabled('dca')) {
    const dcaStrategy = new DCAStrategy(client, config);

    console.log('Strategy: Automated token purchases at regular intervals');
    console.log('Benefit: Reduces impact of price volatility\n');

    try {
      // Execute DCA purchase
      const dcaResult = await dcaStrategy.execute();

      // Get DCA statistics
      const stats = await dcaStrategy.getStats();
      console.log('\nDCA Statistics:');
      console.log(`  Total Purchases: ${stats.totalPurchases}`);
      console.log(`  Successful: ${stats.successfulPurchases}`);
      console.log(`  Failed: ${stats.failedPurchases}`);
      console.log(`  Total Spent: ${stats.totalSpent}\n`);
    } catch (error) {
      console.error('DCA execution error:', error);
    }
  } else {
    console.log('⚠️  DCA strategy is disabled in config\n');
  }

  // ==================== Demo 4: Portfolio Rebalancing ====================

  console.log('═══════════════════════════════════════════════════════');
  console.log('Demo 4: Portfolio Rebalancing Strategy');
  console.log('═══════════════════════════════════════════════════════\n');

  if (config.isStrategyEnabled('rebalance')) {
    const rebalanceStrategy = new RebalanceStrategy(client, config);

    console.log('Strategy: Maintain target asset allocations');
    console.log('Benefit: Automated portfolio management\n');

    try {
      // Analyze portfolio
      const analysis = await rebalanceStrategy.analyzePortfolio();

      if (analysis.needsRebalance) {
        console.log('Portfolio needs rebalancing!');
        console.log(`Planned Actions: ${analysis.actions.length}\n`);

        // Execute rebalancing (commented out for safety)
        // const results = await rebalanceStrategy.execute();
      } else {
        console.log('✅ Portfolio is balanced\n');
      }
    } catch (error) {
      console.error('Rebalance analysis error:', error);
    }
  } else {
    console.log('⚠️  Rebalance strategy is disabled in config\n');
  }

  // ==================== Demo 5: Audit Logs ====================

  console.log('═══════════════════════════════════════════════════════');
  console.log('Demo 5: Audit Trail Inspection');
  console.log('═══════════════════════════════════════════════════════\n');

  try {
    const auditLogs = await client.getAuditLogs();

    console.log(`Total Audit Entries: ${auditLogs.length}`);

    if (auditLogs.length > 0) {
      console.log('\nRecent Actions:');
      auditLogs.slice(-5).forEach(log => {
        console.log(`  [${new Date(Number(log.timestamp) / 1000000).toISOString()}]`);
        console.log(`    Type: ${log.action_type}`);
        console.log(`    Requester: ${log.requester.toText()}`);
        console.log(`    Decision: ${Object.keys(log.policy_result.decision)[0]}`);

        if (log.execution_result.length > 0) {
          const result = log.execution_result[0];
          console.log(`    Success: ${result?.success ? '✅' : '❌'}`);
          if (result?.tx_hash && result.tx_hash.length > 0) {
            console.log(`    TX: ${result.tx_hash[0]}`);
          }
        }
        console.log('');
      });
    }
  } catch (error) {
    console.error('Error fetching audit logs:', error);
  }

  // ==================== Demo 6: System Info ====================

  console.log('═══════════════════════════════════════════════════════');
  console.log('Demo 6: System Information');
  console.log('═══════════════════════════════════════════════════════\n');

  try {
    const isPaused = await client.isPaused();
    console.log(`System Status: ${isPaused ? '⏸️  Paused' : '▶️  Active'}`);

    const systemConfig = await client.getConfig();
    if (systemConfig) {
      console.log(`Canister Name: ${systemConfig.name}`);
      console.log(`Supported Chains: ${systemConfig.supported_chains.join(', ')}`);
      console.log(`Active Policies: ${systemConfig.policies.length}`);
      console.log(
        `Default Threshold: ${systemConfig.default_threshold.required}/${systemConfig.default_threshold.total}`
      );
    }

    const roles = await client.getRoles(client.getPrincipal());
    console.log(`Your Roles: ${roles.map(r => Object.keys(r)[0]).join(', ') || 'None'}\n`);
  } catch (error) {
    console.error('Error fetching system info:', error);
  }

  // ==================== Summary ====================

  console.log('\n╔═══════════════════════════════════════════════════════╗');
  console.log('║   Demo Complete!                                      ║');
  console.log('╚═══════════════════════════════════════════════════════╝\n');

  console.log('Key Features Demonstrated:');
  console.log('  ✅ Policy-based access control');
  console.log('  ✅ Threshold signature workflow');
  console.log('  ✅ Automated DCA strategy');
  console.log('  ✅ Portfolio rebalancing');
  console.log('  ✅ Complete audit trail');
  console.log('  ✅ System monitoring\n');

  console.log('Next Steps:');
  console.log('  1. Configure your identity (PEM file)');
  console.log('  2. Customize strategies in config.yaml');
  console.log('  3. Set up cron schedules for automation');
  console.log('  4. Monitor audit logs for compliance\n');
}

if (require.main === module) {
  main().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}
