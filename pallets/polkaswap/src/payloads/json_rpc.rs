use alt_serde::{Serialize};

// Struct for making Ethereum JSON RPC requests
#[serde(crate = "alt_serde")]
#[derive(Serialize)]
pub struct JSONRpcRequest<T> {
	pub jsonrpc: &'static str,
	pub method: &'static str,
	pub params: T,
	pub id: u32,
}
