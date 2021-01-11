use sp_std::prelude::*;
// We use `alt_serde`, and Xanewok-modified `serde_json` so that we can compile the program
// with serde(features `std`) and alt_serde(features `no_std`).
use alt_serde::Deserialize;
use super::serde_helpers::*;

// Payload for JSON RPCRequest to get the last block of Ethereum network
#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub struct EthBlockNumberResponse {
	// #[serde(deserialize_with = "de_string_to_bytes")]
	// jsonrpc: Vec<u8>,

	// id: u32,

	#[serde(deserialize_with = "de_hex_to_u32")]
	pub(crate) result: u32,
}
