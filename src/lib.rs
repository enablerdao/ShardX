pub mod ai;
pub mod api;
pub mod api_handlers;
pub mod async_runtime;
pub mod async_utils; // asyncは予約語なので変更
pub mod chart;
pub mod consensus;
pub mod cross_chain;
pub mod cross_shard;
pub mod crypto;
pub mod dex;
pub mod error;
pub mod memory;
pub mod metrics;
pub mod multisig;
pub mod node;
pub mod parallel;
pub mod rpc;
pub mod shard;
pub mod sharding;
pub mod smart_contract;
pub mod storage;
pub mod transaction;
pub mod transaction_analysis;
pub mod visualization;
pub mod wallet;

#[cfg(test)]
mod tests;

// Re-export key types for easier access
pub use cross_chain::{BridgeConfig, ChainType, CrossChainBridge};

// Re-export external crates for internal use
#[cfg(feature = "snow")]
pub(crate) use snow;
