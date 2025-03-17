pub mod ai;
pub mod api;
pub mod api_handlers;
pub mod async_utils; // asyncは予約語なので変更
pub mod async_runtime;
pub mod chart;
pub mod consensus;
pub mod cross_shard;
pub mod cross_chain;
pub mod crypto;
pub mod dex;
pub mod error;
pub mod memory;
pub mod multisig;
pub mod node;
pub mod parallel;
pub mod rpc;
pub mod shard;
pub mod sharding;
pub mod storage;
pub mod transaction;
pub mod transaction_analysis;
pub mod visualization;
pub mod wallet;

#[cfg(test)]
mod tests;

// Re-export key types for easier access
pub use cross_chain::{CrossChainBridge, BridgeConfig, ChainType};