/**
 * Configuration management for AI Agent
 */

import * as fs from 'fs';
import * as yaml from 'js-yaml';

export interface TokenConfig {
  symbol: string;
  address: string;
  decimals: number;
}

export interface ChainConfig {
  name: string;
  tokens: Record<string, TokenConfig>;
}

export interface StrategyConfig {
  enabled: boolean;
  interval?: string; // Cron format
  params?: Record<string, any>;
}

export interface AgentConfig {
  // ChainGuard canister settings
  canister: {
    id: string;
    network: 'ic' | 'local';
    host?: string;
  };

  // Identity settings
  identity: {
    type: 'pem' | 'anonymous';
    path?: string;
  };

  // Chain configurations
  chains: Record<string, ChainConfig>;

  // Strategy configurations
  strategies: {
    dca?: StrategyConfig & {
      params: {
        sourceToken: string;
        targetToken: string;
        amountPerPurchase: string; // In source token units
        chain: string;
        minAmountOut?: string; // Minimum slippage protection
      };
    };
    rebalance?: StrategyConfig & {
      params: {
        portfolio: Array<{
          token: string;
          targetPercentage: number;
        }>;
        chain: string;
        rebalanceThreshold: number; // Percentage deviation to trigger rebalance
      };
    };
  };

  // Risk management
  limits: {
    maxTransactionSize: string; // In wei/smallest unit
    dailyLimit: string;
  };
}

export class ConfigManager {
  private config: AgentConfig;

  constructor(configPath: string) {
    this.config = this.loadConfig(configPath);
    this.validateConfig();
  }

  private loadConfig(path: string): AgentConfig {
    const fileContents = fs.readFileSync(path, 'utf8');

    if (path.endsWith('.yaml') || path.endsWith('.yml')) {
      return yaml.load(fileContents) as AgentConfig;
    } else if (path.endsWith('.json')) {
      return JSON.parse(fileContents);
    } else {
      throw new Error('Unsupported config format. Use .yaml, .yml, or .json');
    }
  }

  private validateConfig(): void {
    if (!this.config.canister?.id) {
      throw new Error('Canister ID is required in config');
    }

    if (!this.config.chains || Object.keys(this.config.chains).length === 0) {
      throw new Error('At least one chain configuration is required');
    }
  }

  getConfig(): AgentConfig {
    return this.config;
  }

  getCanisterConfig() {
    return this.config.canister;
  }

  getIdentityConfig() {
    return this.config.identity;
  }

  getChainConfig(chainName: string): ChainConfig | undefined {
    return this.config.chains[chainName];
  }

  getTokenAddress(chain: string, symbol: string): string | undefined {
    const chainConfig = this.getChainConfig(chain);
    return chainConfig?.tokens[symbol]?.address;
  }

  getStrategyConfig<T extends keyof AgentConfig['strategies']>(
    strategy: T
  ): AgentConfig['strategies'][T] {
    return this.config.strategies[strategy];
  }

  isStrategyEnabled(strategy: keyof AgentConfig['strategies']): boolean {
    return this.config.strategies[strategy]?.enabled ?? false;
  }
}
