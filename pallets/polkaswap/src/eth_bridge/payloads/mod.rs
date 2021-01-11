/// ETH_BRIDGE PAYLOADS
/// This module contains entities structures and ser
pub mod eth_block;
pub mod serde_helpers;
pub mod eth_get_logs;
pub mod json_rpc;
pub mod erc20_transfer;

pub use eth_block::EthBlockNumberResponse;
pub use eth_get_logs::{EthGetLogsRequest, EthGetLogsResponse, FromTxLog, TxLog};
pub use json_rpc::JSONRpcRequest;
pub use erc20_transfer::ERC20Event;
