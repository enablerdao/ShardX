pub mod ai;
pub mod api;
pub mod api_handlers;
#[allow(unused_imports)]
pub mod r#async;
pub mod async_runtime;
pub mod consensus;
pub mod cross_shard;
pub mod crypto;
pub mod dex;
pub mod error;
pub mod memory;
pub mod multisig;
pub mod node;
pub mod parallel;
pub mod rpc;
pub mod shard;
// pub mod sharding; // Commented out to resolve module conflict
pub mod storage;
pub mod transaction;
pub mod transaction_analysis;
pub mod visualization;
pub mod wallet;

#[cfg(test)]
mod tests;
