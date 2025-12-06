#!/bin/bash

# ChainGuard SDK - Initialization Example

set -e

export PATH="$HOME/.local/share/dfx/bin:$PATH"

echo "🔧 Initializing ChainGuard with example configuration..."

dfx canister call chainguard initialize '(
  record {
    name = "ChainGuard Demo Instance";
    default_threshold = record { required = 2; total = 3 };
    supported_chains = vec { "ethereum"; "polygon"; "arbitrum" };
    policies = vec {
      record {
        name = "Allow Small Transfers";
        conditions = vec { variant { MaxAmount = 1000000000 } };
        action = variant { Allow };
        priority = 1
      };
      record {
        name = "Require Threshold for Large Transfers";
        conditions = vec { variant { MaxAmount = 10000000000 } };
        action = variant { RequireThreshold = record { required = 2; from_roles = vec { variant { Owner }; variant { Operator } } } };
        priority = 2
      };
      record {
        name = "Block Very Large Transfers";
        conditions = vec { variant { MaxAmount = 100000000000 } };
        action = variant { Deny };
        priority = 0
      }
    }
  }
)'

echo ""
echo "✅ Initialization complete!"
echo ""
echo "Policies configured:"
echo "1. Allow transfers < 1B units"
echo "2. Require 2-of-N signatures for transfers < 10B units"
echo "3. Block transfers > 100B units"
