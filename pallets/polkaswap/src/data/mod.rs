pub use block_event::BlockEvents;
pub use contract_method::ContractMethod;
pub use eth_address::EthAddress;
pub use hash256::Hash256;
pub use sender_amount::SenderAmount;
pub use uint256::Uint256;

pub use crate::eth_bridge::payloads::erc20_transfer::ERC20Event;

pub mod contract_method;
pub mod sender_amount;
mod block_event;
pub mod eth_address;
pub mod hash256;
pub mod uint256;

