/// Configuration template for ChainGuard
///
/// To use this:
/// 1. Copy this file to `config.rs` in the same directory
/// 2. Replace YOUR_ALCHEMY_API_KEY with your actual Alchemy API key
/// 3. The `config.rs` file is ignored by git for security

/// Alchemy API key for Sepolia RPC
pub const ALCHEMY_API_KEY: &str = "YOUR_ALCHEMY_API_KEY";

/// Construct the Alchemy Sepolia RPC URL
pub fn get_alchemy_sepolia_url() -> String {
    format!("https://eth-sepolia.g.alchemy.com/v2/{}", ALCHEMY_API_KEY)
}
