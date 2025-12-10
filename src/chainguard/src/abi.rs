/// ABI encoding utilities for ERC20 and Uniswap V2 contract interactions
use ethers_core::types::{Address, U256};

/// ERC20 function selectors (first 4 bytes of keccak256 hash of signature)
pub mod erc20 {
    use super::*;

    /// approve(address,uint256) selector: 0x095ea7b3
    pub const APPROVE_SELECTOR: [u8; 4] = [0x09, 0x5e, 0xa7, 0xb3];

    /// transfer(address,uint256) selector: 0xa9059cbb
    pub const TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

    /// balanceOf(address) selector: 0x70a08231
    pub const BALANCE_OF_SELECTOR: [u8; 4] = [0x70, 0xa0, 0x82, 0x31];

    /// allowance(address,address) selector: 0xdd62ed3e
    pub const ALLOWANCE_SELECTOR: [u8; 4] = [0xdd, 0x62, 0xed, 0x3e];

    /// Encode approve(address spender, uint256 amount) call data
    pub fn encode_approve(spender: Address, amount: U256) -> Vec<u8> {
        let mut data = Vec::with_capacity(68); // 4 + 32 + 32

        // Function selector
        data.extend_from_slice(&APPROVE_SELECTOR);

        // Pad address to 32 bytes (left-padded with zeros)
        let mut padded_address = [0u8; 32];
        padded_address[12..32].copy_from_slice(spender.as_bytes());
        data.extend_from_slice(&padded_address);

        // Amount as 32-byte big-endian
        let mut amount_bytes = [0u8; 32];
        amount.to_big_endian(&mut amount_bytes);
        data.extend_from_slice(&amount_bytes);

        data
    }

    /// Encode transfer(address to, uint256 amount) call data
    pub fn encode_transfer(to: Address, amount: U256) -> Vec<u8> {
        let mut data = Vec::with_capacity(68);

        data.extend_from_slice(&TRANSFER_SELECTOR);

        let mut padded_address = [0u8; 32];
        padded_address[12..32].copy_from_slice(to.as_bytes());
        data.extend_from_slice(&padded_address);

        let mut amount_bytes = [0u8; 32];
        amount.to_big_endian(&mut amount_bytes);
        data.extend_from_slice(&amount_bytes);

        data
    }

    /// Encode balanceOf(address account) call data
    pub fn encode_balance_of(account: Address) -> Vec<u8> {
        let mut data = Vec::with_capacity(36);

        data.extend_from_slice(&BALANCE_OF_SELECTOR);

        let mut padded_address = [0u8; 32];
        padded_address[12..32].copy_from_slice(account.as_bytes());
        data.extend_from_slice(&padded_address);

        data
    }

    /// Encode allowance(address owner, address spender) call data
    pub fn encode_allowance(owner: Address, spender: Address) -> Vec<u8> {
        let mut data = Vec::with_capacity(68);

        data.extend_from_slice(&ALLOWANCE_SELECTOR);

        let mut padded_owner = [0u8; 32];
        padded_owner[12..32].copy_from_slice(owner.as_bytes());
        data.extend_from_slice(&padded_owner);

        let mut padded_spender = [0u8; 32];
        padded_spender[12..32].copy_from_slice(spender.as_bytes());
        data.extend_from_slice(&padded_spender);

        data
    }
}

/// WETH9 function selectors and encoding
pub mod weth {
    /// deposit() selector: 0xd0e30db0
    /// Converts ETH to WETH by sending ETH as msg.value
    pub const DEPOSIT_SELECTOR: [u8; 4] = [0xd0, 0xe3, 0x0d, 0xb0];

    /// Encode deposit() call data (no parameters, just selector)
    pub fn encode_deposit() -> Vec<u8> {
        DEPOSIT_SELECTOR.to_vec()
    }
}

/// Uniswap V2 Router02 function selectors and encoding
pub mod uniswap_v2 {
    use super::*;

    /// swapExactTokensForTokens(uint256,uint256,address[],address,uint256) selector: 0x38ed1739
    pub const SWAP_EXACT_TOKENS_FOR_TOKENS: [u8; 4] = [0x38, 0xed, 0x17, 0x39];

    /// swapExactETHForTokens(uint256,address[],address,uint256) selector: 0x7ff36ab5
    pub const SWAP_EXACT_ETH_FOR_TOKENS: [u8; 4] = [0x7f, 0xf3, 0x6a, 0xb5];

    /// swapExactTokensForETH(uint256,uint256,address[],address,uint256) selector: 0x18cbafe5
    pub const SWAP_EXACT_TOKENS_FOR_ETH: [u8; 4] = [0x18, 0xcb, 0xaf, 0xe5];

    /// Encode swapExactTokensForTokens call data
    /// function swapExactTokensForTokens(
    ///   uint amountIn,
    ///   uint amountOutMin,
    ///   address[] calldata path,
    ///   address to,
    ///   uint deadline
    /// )
    pub fn encode_swap_exact_tokens_for_tokens(
        amount_in: U256,
        amount_out_min: U256,
        path: Vec<Address>,
        to: Address,
        deadline: U256,
    ) -> Vec<u8> {
        let mut data = Vec::new();

        // Function selector
        data.extend_from_slice(&SWAP_EXACT_TOKENS_FOR_TOKENS);

        // amountIn (uint256)
        let mut amount_in_bytes = [0u8; 32];
        amount_in.to_big_endian(&mut amount_in_bytes);
        data.extend_from_slice(&amount_in_bytes);

        // amountOutMin (uint256)
        let mut amount_out_min_bytes = [0u8; 32];
        amount_out_min.to_big_endian(&mut amount_out_min_bytes);
        data.extend_from_slice(&amount_out_min_bytes);

        // Offset to path array (0xa0 = 160 bytes from start of data)
        let path_offset = [0u8; 31];
        data.extend_from_slice(&path_offset);
        data.push(0xa0);

        // to (address) - padded to 32 bytes
        let mut padded_to = [0u8; 32];
        padded_to[12..32].copy_from_slice(to.as_bytes());
        data.extend_from_slice(&padded_to);

        // deadline (uint256)
        let mut deadline_bytes = [0u8; 32];
        deadline.to_big_endian(&mut deadline_bytes);
        data.extend_from_slice(&deadline_bytes);

        // path array length
        let path_len = U256::from(path.len());
        let mut path_len_bytes = [0u8; 32];
        path_len.to_big_endian(&mut path_len_bytes);
        data.extend_from_slice(&path_len_bytes);

        // path array elements
        for addr in path {
            let mut padded_addr = [0u8; 32];
            padded_addr[12..32].copy_from_slice(addr.as_bytes());
            data.extend_from_slice(&padded_addr);
        }

        data
    }

    /// Encode swapExactETHForTokens call data
    /// function swapExactETHForTokens(
    ///   uint amountOutMin,
    ///   address[] calldata path,
    ///   address to,
    ///   uint deadline
    /// )
    pub fn encode_swap_exact_eth_for_tokens(
        amount_out_min: U256,
        path: Vec<Address>,
        to: Address,
        deadline: U256,
    ) -> Vec<u8> {
        let mut data = Vec::new();

        // Function selector
        data.extend_from_slice(&SWAP_EXACT_ETH_FOR_TOKENS);

        // amountOutMin (uint256)
        let mut amount_out_min_bytes = [0u8; 32];
        amount_out_min.to_big_endian(&mut amount_out_min_bytes);
        data.extend_from_slice(&amount_out_min_bytes);

        // Offset to path array (0x80 = 128 bytes from start of data)
        let path_offset = [0u8; 31];
        data.extend_from_slice(&path_offset);
        data.push(0x80);

        // to (address) - padded to 32 bytes
        let mut padded_to = [0u8; 32];
        padded_to[12..32].copy_from_slice(to.as_bytes());
        data.extend_from_slice(&padded_to);

        // deadline (uint256)
        let mut deadline_bytes = [0u8; 32];
        deadline.to_big_endian(&mut deadline_bytes);
        data.extend_from_slice(&deadline_bytes);

        // path array length
        let path_len = U256::from(path.len());
        let mut path_len_bytes = [0u8; 32];
        path_len.to_big_endian(&mut path_len_bytes);
        data.extend_from_slice(&path_len_bytes);

        // path array elements
        for addr in path {
            let mut padded_addr = [0u8; 32];
            padded_addr[12..32].copy_from_slice(addr.as_bytes());
            data.extend_from_slice(&padded_addr);
        }

        data
    }

    /// Encode swapExactTokensForETH call data
    /// function swapExactTokensForETH(
    ///   uint amountIn,
    ///   uint amountOutMin,
    ///   address[] calldata path,
    ///   address to,
    ///   uint deadline
    /// )
    pub fn encode_swap_exact_tokens_for_eth(
        amount_in: U256,
        amount_out_min: U256,
        path: Vec<Address>,
        to: Address,
        deadline: U256,
    ) -> Vec<u8> {
        let mut data = Vec::new();

        // Function selector
        data.extend_from_slice(&SWAP_EXACT_TOKENS_FOR_ETH);

        // amountIn (uint256)
        let mut amount_in_bytes = [0u8; 32];
        amount_in.to_big_endian(&mut amount_in_bytes);
        data.extend_from_slice(&amount_in_bytes);

        // amountOutMin (uint256)
        let mut amount_out_min_bytes = [0u8; 32];
        amount_out_min.to_big_endian(&mut amount_out_min_bytes);
        data.extend_from_slice(&amount_out_min_bytes);

        // Offset to path array (0xa0 = 160 bytes from start of data)
        let path_offset = [0u8; 31];
        data.extend_from_slice(&path_offset);
        data.push(0xa0);

        // to (address) - padded to 32 bytes
        let mut padded_to = [0u8; 32];
        padded_to[12..32].copy_from_slice(to.as_bytes());
        data.extend_from_slice(&padded_to);

        // deadline (uint256)
        let mut deadline_bytes = [0u8; 32];
        deadline.to_big_endian(&mut deadline_bytes);
        data.extend_from_slice(&deadline_bytes);

        // path array length
        let path_len = U256::from(path.len());
        let mut path_len_bytes = [0u8; 32];
        path_len.to_big_endian(&mut path_len_bytes);
        data.extend_from_slice(&path_len_bytes);

        // path array elements
        for addr in path {
            let mut padded_addr = [0u8; 32];
            padded_addr[12..32].copy_from_slice(addr.as_bytes());
            data.extend_from_slice(&padded_addr);
        }

        data
    }
}

/// Uniswap V3 SwapRouter function selectors and encoding
pub mod uniswap_v3 {
    use super::*;

    /// exactInputSingle((address,address,uint24,address,uint256,uint256,uint160)) selector: 0x04e45aaf
    pub const EXACT_INPUT_SINGLE_SELECTOR: [u8; 4] = [0x04, 0xe4, 0x5a, 0xaf];

    /// Encode exactInputSingle call data for Uniswap V3
    /// function exactInputSingle(ExactInputSingleParams calldata params) external payable returns (uint256 amountOut)
    /// struct ExactInputSingleParams {
    ///   address tokenIn;
    ///   address tokenOut;
    ///   uint24 fee;
    ///   address recipient;
    ///   uint256 deadline;
    ///   uint256 amountIn;
    ///   uint256 amountOutMinimum;
    ///   uint160 sqrtPriceLimitX96;
    /// }
    pub fn encode_exact_input_single(
        token_in: Address,
        token_out: Address,
        fee: u32,        // 500 = 0.05%, 3000 = 0.3%, 10000 = 1%
        recipient: Address,
        amount_in: U256,
        amount_out_minimum: U256,
        sqrt_price_limit_x96: U256,
    ) -> Vec<u8> {
        let mut data = Vec::new();

        // Function selector
        data.extend_from_slice(&EXACT_INPUT_SINGLE_SELECTOR);

        // Parameters go directly after selector (no offset for non-tuple params)

        // tokenIn (address) - padded to 32 bytes
        let mut padded_token_in = [0u8; 32];
        padded_token_in[12..32].copy_from_slice(token_in.as_bytes());
        data.extend_from_slice(&padded_token_in);

        // tokenOut (address) - padded to 32 bytes
        let mut padded_token_out = [0u8; 32];
        padded_token_out[12..32].copy_from_slice(token_out.as_bytes());
        data.extend_from_slice(&padded_token_out);

        // fee (uint24) - padded to 32 bytes
        let mut fee_bytes = [0u8; 32];
        fee_bytes[28..32].copy_from_slice(&fee.to_be_bytes());
        data.extend_from_slice(&fee_bytes);

        // recipient (address) - padded to 32 bytes
        let mut padded_recipient = [0u8; 32];
        padded_recipient[12..32].copy_from_slice(recipient.as_bytes());
        data.extend_from_slice(&padded_recipient);

        // amountIn (uint256)
        let mut amount_in_bytes = [0u8; 32];
        amount_in.to_big_endian(&mut amount_in_bytes);
        data.extend_from_slice(&amount_in_bytes);

        // amountOutMinimum (uint256)
        let mut amount_out_min_bytes = [0u8; 32];
        amount_out_minimum.to_big_endian(&mut amount_out_min_bytes);
        data.extend_from_slice(&amount_out_min_bytes);

        // sqrtPriceLimitX96 (uint160) - padded to 32 bytes
        let mut sqrt_price_bytes = [0u8; 32];
        sqrt_price_limit_x96.to_big_endian(&mut sqrt_price_bytes);
        data.extend_from_slice(&sqrt_price_bytes);

        data
    }
}

/// Permit2 AllowanceTransfer functions
pub mod permit2 {
    use super::*;

    /// approve(address,address,uint160,uint48) selector: 0x87517c45
    pub const APPROVE_SELECTOR: [u8; 4] = [0x87, 0x51, 0x7c, 0x45];

    /// Encode approve(address token, address spender, uint160 amount, uint48 expiration) call data
    /// This is the second approval needed for Permit2 AllowanceTransfer
    pub fn encode_approve(
        token: Address,
        spender: Address,
        amount: U256,
        expiration: u64,
    ) -> Vec<u8> {
        let mut data = Vec::with_capacity(132); // 4 + 32 + 32 + 32 + 32

        // Function selector
        data.extend_from_slice(&APPROVE_SELECTOR);

        // token (address) - padded to 32 bytes
        let mut padded_token = [0u8; 32];
        padded_token[12..32].copy_from_slice(token.as_bytes());
        data.extend_from_slice(&padded_token);

        // spender (address) - padded to 32 bytes
        let mut padded_spender = [0u8; 32];
        padded_spender[12..32].copy_from_slice(spender.as_bytes());
        data.extend_from_slice(&padded_spender);

        // amount (uint160) - padded to 32 bytes
        // Note: uint160 is 20 bytes, but we pad to 32 for ABI encoding
        let mut amount_bytes = [0u8; 32];
        amount.to_big_endian(&mut amount_bytes);
        data.extend_from_slice(&amount_bytes);

        // expiration (uint48) - padded to 32 bytes
        // Note: uint48 is 6 bytes, but we pad to 32 for ABI encoding
        let mut expiration_bytes = [0u8; 32];
        let expiration_u256 = U256::from(expiration);
        expiration_u256.to_big_endian(&mut expiration_bytes);
        data.extend_from_slice(&expiration_bytes);

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approve_encoding() {
        // Test data
        let spender: Address = "0x1111111111111111111111111111111111111111"
            .parse()
            .unwrap();
        let amount = U256::from(1000000u64);

        let encoded = erc20::encode_approve(spender, amount);

        // Should be 68 bytes: 4 (selector) + 32 (address) + 32 (amount)
        assert_eq!(encoded.len(), 68);

        // First 4 bytes should be approve selector
        assert_eq!(&encoded[0..4], &erc20::APPROVE_SELECTOR);

        // Verify the encoding format
        assert_eq!(encoded[0], 0x09);
        assert_eq!(encoded[1], 0x5e);
        assert_eq!(encoded[2], 0xa7);
        assert_eq!(encoded[3], 0xb3);
    }

    #[test]
    fn test_transfer_encoding() {
        let to: Address = "0x2222222222222222222222222222222222222222"
            .parse()
            .unwrap();
        let amount = U256::from(500000u64);

        let encoded = erc20::encode_transfer(to, amount);

        assert_eq!(encoded.len(), 68);
        assert_eq!(&encoded[0..4], &erc20::TRANSFER_SELECTOR);
    }

    #[test]
    fn test_balance_of_encoding() {
        let account: Address = "0x3333333333333333333333333333333333333333"
            .parse()
            .unwrap();

        let encoded = erc20::encode_balance_of(account);

        assert_eq!(encoded.len(), 36); // 4 + 32
        assert_eq!(&encoded[0..4], &erc20::BALANCE_OF_SELECTOR);
    }

    #[test]
    fn test_allowance_encoding() {
        let owner: Address = "0x4444444444444444444444444444444444444444"
            .parse()
            .unwrap();
        let spender: Address = "0x5555555555555555555555555555555555555555"
            .parse()
            .unwrap();

        let encoded = erc20::encode_allowance(owner, spender);

        assert_eq!(encoded.len(), 68);
        assert_eq!(&encoded[0..4], &erc20::ALLOWANCE_SELECTOR);
    }
}
