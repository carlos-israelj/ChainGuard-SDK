/// Universal Router integration for Uniswap swaps
/// Uses command-based execution system with Permit2 for token approvals
use ethers_core::types::{Address, U256};

/// Universal Router command codes
pub mod commands {
    /// V3 swap with exact input amount
    pub const V3_SWAP_EXACT_IN: u8 = 0x00;

    /// V3 swap with exact output amount
    pub const V3_SWAP_EXACT_OUT: u8 = 0x01;

    /// Transfer tokens using Permit2
    pub const PERMIT2_TRANSFER_FROM: u8 = 0x02;

    /// Wrap ETH to WETH
    pub const WRAP_ETH: u8 = 0x0b;

    /// Unwrap WETH to ETH
    pub const UNWRAP_WETH: u8 = 0x0c;

    /// Flag to allow command revert without reverting entire transaction
    pub const FLAG_ALLOW_REVERT: u8 = 0x80;
}

/// Universal Router contract addresses by chain
pub fn get_universal_router_address(chain: &str) -> Option<&'static str> {
    match chain.to_lowercase().as_str() {
        "sepolia" => Some("0x3a9d48ab9751398bbfa63ad67599bb04e4bdf98b"),
        "ethereum" | "mainnet" => Some("0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD"),
        _ => None,
    }
}

/// Permit2 contract address (same on all networks)
pub const PERMIT2_ADDRESS: &str = "0x000000000022D473030F116dDEE9F6B43aC78BA3";

/// Encode a V3 swap path for Uniswap V3
/// Path format: [token0, fee0, token1, fee1, token2, ...]
/// Each fee is 3 bytes (uint24)
pub fn encode_v3_path(tokens: Vec<Address>, fees: Vec<u32>) -> Vec<u8> {
    if tokens.len() != fees.len() + 1 {
        panic!("Invalid path: tokens length must be fees length + 1");
    }

    let mut path = Vec::new();

    for i in 0..fees.len() {
        // Add token address (20 bytes)
        path.extend_from_slice(tokens[i].as_bytes());

        // Add fee (3 bytes, uint24)
        let fee_bytes = fees[i].to_be_bytes();
        path.extend_from_slice(&fee_bytes[1..4]); // Take last 3 bytes
    }

    // Add final token
    path.extend_from_slice(tokens[tokens.len() - 1].as_bytes());

    path
}

/// Encode V3_SWAP_EXACT_IN command input
///
/// Parameters for V3_SWAP_EXACT_IN (from Universal Router docs):
/// - recipient: address - where to send output tokens
/// - amountIn: uint256 - amount of input tokens
/// - amountOutMinimum: uint256 - minimum output tokens (slippage protection)
/// - path: bytes - encoded V3 path (tokens + fees)
/// - payerIsUser: bool - if true, tokens come from msg.sender via Permit2
///                       if false, tokens are already in the router
pub fn encode_v3_swap_exact_in(
    recipient: Address,
    amount_in: U256,
    amount_out_minimum: U256,
    path: Vec<u8>,
    payer_is_user: bool,
) -> Vec<u8> {
    let mut data = Vec::new();

    // Recipient (address, padded to 32 bytes)
    let mut padded_recipient = [0u8; 32];
    padded_recipient[12..32].copy_from_slice(recipient.as_bytes());
    data.extend_from_slice(&padded_recipient);

    // Amount in (uint256)
    let mut amount_in_bytes = [0u8; 32];
    amount_in.to_big_endian(&mut amount_in_bytes);
    data.extend_from_slice(&amount_in_bytes);

    // Amount out minimum (uint256)
    let mut amount_out_min_bytes = [0u8; 32];
    amount_out_minimum.to_big_endian(&mut amount_out_min_bytes);
    data.extend_from_slice(&amount_out_min_bytes);

    // Offset to path bytes array (pointer to where path data starts)
    // Offset = 0xa0 (160 bytes) = 32*5 (recipient + amountIn + amountOutMin + path_offset + payerIsUser)
    let path_offset = [0u8; 31];
    data.extend_from_slice(&path_offset);
    data.push(0xa0);

    // Payer is user (bool, padded to 32 bytes)
    let mut payer_bytes = [0u8; 32];
    payer_bytes[31] = if payer_is_user { 1 } else { 0 };
    data.extend_from_slice(&payer_bytes);

    // Path data (dynamic bytes array)
    // First: length of path in bytes
    let path_len = U256::from(path.len());
    let mut path_len_bytes = [0u8; 32];
    path_len.to_big_endian(&mut path_len_bytes);
    data.extend_from_slice(&path_len_bytes);

    // Then: actual path data
    data.extend_from_slice(&path);

    // Pad to 32-byte boundary if needed
    let padding_needed = (32 - (path.len() % 32)) % 32;
    data.extend_from_slice(&vec![0u8; padding_needed]);

    data
}

/// Encode WRAP_ETH command input
/// Wraps ETH to WETH
///
/// Parameters:
/// - recipient: address - where to send WETH (use ADDRESS_THIS for router)
/// - amountMin: uint256 - minimum amount to wrap
pub fn encode_wrap_eth(recipient: Address, amount_min: U256) -> Vec<u8> {
    let mut data = Vec::new();

    // Recipient (address, padded to 32 bytes)
    let mut padded_recipient = [0u8; 32];
    padded_recipient[12..32].copy_from_slice(recipient.as_bytes());
    data.extend_from_slice(&padded_recipient);

    // Amount min (uint256)
    let mut amount_min_bytes = [0u8; 32];
    amount_min.to_big_endian(&mut amount_min_bytes);
    data.extend_from_slice(&amount_min_bytes);

    data
}

/// Encode UNWRAP_WETH command input
/// Unwraps WETH to ETH
///
/// Parameters:
/// - recipient: address - where to send ETH
/// - amountMin: uint256 - minimum amount to unwrap
pub fn encode_unwrap_weth(recipient: Address, amount_min: U256) -> Vec<u8> {
    let mut data = Vec::new();

    // Recipient (address, padded to 32 bytes)
    let mut padded_recipient = [0u8; 32];
    padded_recipient[12..32].copy_from_slice(recipient.as_bytes());
    data.extend_from_slice(&padded_recipient);

    // Amount min (uint256)
    let mut amount_min_bytes = [0u8; 32];
    amount_min.to_big_endian(&mut amount_min_bytes);
    data.extend_from_slice(&amount_min_bytes);

    data
}

/// Special addresses used by Universal Router
pub mod special_addresses {
    /// Placeholder meaning "use the Universal Router contract itself"
    pub const ADDRESS_THIS: &str = "0x0000000000000000000000000000000000000002";

    /// Placeholder meaning "use msg.sender"
    pub const MSG_SENDER: &str = "0x0000000000000000000000000000000000000001";
}

/// Build complete Universal Router execute() calldata
///
/// The execute function signature is:
/// execute(bytes commands, bytes[] inputs, uint256 deadline)
///
/// Parameters:
/// - commands: each byte is a command code
/// - inputs: array of ABI-encoded inputs for each command
/// - deadline: transaction deadline timestamp
pub fn encode_execute(
    commands: Vec<u8>,
    inputs: Vec<Vec<u8>>,
    deadline: u64,
) -> Vec<u8> {
    if commands.len() != inputs.len() {
        panic!("Commands and inputs must have same length");
    }

    let mut data = Vec::new();

    // Function selector for execute(bytes,bytes[],uint256)
    // keccak256("execute(bytes,bytes[],uint256)") = 0x3593564c...
    let execute_selector: [u8; 4] = [0x35, 0x93, 0x56, 0x4c];
    data.extend_from_slice(&execute_selector);

    // Offset to commands bytes (0x60 = 96 bytes)
    // This is: 32 (commands offset) + 32 (inputs offset) + 32 (deadline)
    let commands_offset = [0u8; 31];
    data.extend_from_slice(&commands_offset);
    data.push(0x60);

    // Offset to inputs array - calculate dynamically
    // inputs_offset = 0x60 + 32 (commands length) + commands.len() + padding
    let commands_padded_len = ((commands.len() + 31) / 32) * 32;
    let inputs_offset_value = 0x60 + 32 + commands_padded_len;
    let mut inputs_offset_bytes = [0u8; 32];
    U256::from(inputs_offset_value).to_big_endian(&mut inputs_offset_bytes);
    data.extend_from_slice(&inputs_offset_bytes);

    // Deadline (uint256)
    let mut deadline_bytes = [0u8; 32];
    U256::from(deadline).to_big_endian(&mut deadline_bytes);
    data.extend_from_slice(&deadline_bytes);

    // Commands bytes
    // Length
    let mut commands_len_bytes = [0u8; 32];
    U256::from(commands.len()).to_big_endian(&mut commands_len_bytes);
    data.extend_from_slice(&commands_len_bytes);

    // Data
    data.extend_from_slice(&commands);

    // Padding
    let commands_padding = commands_padded_len - commands.len();
    data.extend_from_slice(&vec![0u8; commands_padding]);

    // Inputs array
    // Array length
    let mut inputs_len_bytes = [0u8; 32];
    U256::from(inputs.len()).to_big_endian(&mut inputs_len_bytes);
    data.extend_from_slice(&inputs_len_bytes);

    // Calculate offsets for each input
    let mut current_offset = inputs.len() * 32; // Start after all offset pointers
    let mut offsets = Vec::new();

    for input in &inputs {
        offsets.push(current_offset);
        let padded_len = ((input.len() + 31) / 32) * 32;
        current_offset += 32 + padded_len; // 32 for length + padded data
    }

    // Write offset pointers
    for offset in offsets {
        let mut offset_bytes = [0u8; 32];
        U256::from(offset).to_big_endian(&mut offset_bytes);
        data.extend_from_slice(&offset_bytes);
    }

    // Write actual input data
    for input in inputs {
        // Length
        let mut input_len_bytes = [0u8; 32];
        U256::from(input.len()).to_big_endian(&mut input_len_bytes);
        data.extend_from_slice(&input_len_bytes);

        // Data
        data.extend_from_slice(&input);

        // Padding
        let padding_needed = (32 - (input.len() % 32)) % 32;
        data.extend_from_slice(&vec![0u8; padding_needed]);
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_v3_path_single_hop() {
        let token0: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse().unwrap(); // WETH
        let token1: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse().unwrap(); // USDC
        let fee = 3000u32; // 0.3%

        let path = encode_v3_path(vec![token0, token1], vec![fee]);

        // Path should be: 20 bytes (token0) + 3 bytes (fee) + 20 bytes (token1) = 43 bytes
        assert_eq!(path.len(), 43);
    }

    #[test]
    fn test_encode_v3_path_multi_hop() {
        let token0: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse().unwrap();
        let token1: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse().unwrap();
        let token2: Address = "0x6B175474E89094C44Da98b954EedeAC495271d0F".parse().unwrap();

        let path = encode_v3_path(
            vec![token0, token1, token2],
            vec![3000, 500],
        );

        // Path should be: 20 + 3 + 20 + 3 + 20 = 66 bytes
        assert_eq!(path.len(), 66);
    }

    #[test]
    fn test_command_codes() {
        assert_eq!(commands::V3_SWAP_EXACT_IN, 0x00);
        assert_eq!(commands::WRAP_ETH, 0x0b);
        assert_eq!(commands::UNWRAP_WETH, 0x0c);
    }
}
