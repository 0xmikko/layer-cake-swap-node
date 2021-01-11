use core::fmt;

// We use `alt_serde`, and Xanewok-modified `serde_json` so that we can compile the program
// with serde(features `std`) and alt_serde(features `no_std`).
use alt_serde::{Deserialize, Serialize};
use ethabi::{Address, Hash, RawLog};

use hex::encode;
use sp_std::fmt::Formatter;
use sp_std::prelude::*;

use super::serde_helpers::*;

#[serde(crate = "alt_serde")]
#[derive(Serialize)]
pub struct EthGetLogsRequest {
	// #[serde(serialize_with = "ser_address_to_hex")]
	// address: Address,

	#[serde(serialize_with = "ser_u32_to_hex")]
	pub(crate) from_block: u32,

	#[serde(serialize_with = "ser_u32_to_hex")]
	pub(crate) to_block: u32,
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub struct TxLog {
	#[serde(deserialize_with = "de_hex_to_address")]
	pub(crate) address: Address,

	#[serde(rename = "blockHash", deserialize_with = "de_hex_to_hash")]
	block_hash: Hash,

	#[serde(rename = "blockNumber", deserialize_with = "de_hex_to_u32")]
	block_number: u32,

	#[serde(deserialize_with = "de_hex_to_vec_u8")]
	pub(crate) data: Vec<u8>,

	#[serde(rename = "logIndex", deserialize_with = "de_hex_to_u32")]
	log_index: u32,

	removed: bool,

	#[serde(deserialize_with = "decode_hex_hash_seq")]
	pub(crate) topics: Vec<Hash>,

	#[serde(rename = "transactionHash", deserialize_with = "de_hex_to_hash")]
	transaction_hash: Hash,

	#[serde(rename = "transactionIndex", deserialize_with = "de_hex_to_u32")]
	transaction_index: u32,
}

impl fmt::Display for TxLog {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let hex_value = encode(self.address.as_bytes());
		let result = ["0x", hex_value.as_str()].concat();
		write!(f, "(from: {})", result.as_str())
	}
}

pub trait FromTxLog {
	fn from_tx(tx: &TxLog) -> Self;
}

impl FromTxLog for RawLog {
	fn from_tx(tx: &TxLog) -> Self {
		RawLog::from((tx.topics.clone(), tx.data.clone()))
	}
}


#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub struct EthGetLogsResponse {
	// #[serde(deserialize_with = "de_string_to_bytes")]
	// jsonrpc: Vec<u8>,
	//
	// id: u32,

	pub(crate) result: Vec<TxLog>,
}
