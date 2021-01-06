use core::{convert::*, fmt};

// We use `alt_serde`, and Xanewok-modified `serde_json` so that we can compile the program
// with serde(features `std`) and alt_serde(features `no_std`).
use alt_serde::{Deserialize, Deserializer, Serialize};
use ethereum_types::{Address, H256};
use frame_support::{debug, traits::Get};
use hex::encode;
use sp_runtime::{
	offchain as rt_offchain,
	RuntimeDebug,
};
use sp_std::str;
use sp_std::fmt::Formatter;
use sp_std::prelude::*;
use sp_std::str::FromStr;

use crate::serde_helpers::*;

use super::{Error, Module, Trait};
use ethabi::Hash;

pub const FETCH_TIMEOUT_PERIOD: u64 = 30000;

// Struct for making Ethereum JSON RPC requests
#[serde(crate = "alt_serde")]
#[derive(Serialize)]
struct JSONRpcRequest<T> {
	jsonrpc: &'static str,
	method: &'static str,
	params: T,
	id: u32,
}

// Payload for JSON RPCRequest to get the last block of Ethereum network
#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub(crate) struct EthBlockNumberResponse {
	#[serde(deserialize_with = "de_string_to_bytes")]
	jsonrpc: Vec<u8>,

	id: u32,

	#[serde(deserialize_with = "de_hex_to_u32")]
	result: u32,
}

#[serde(crate = "alt_serde")]
#[derive(Serialize)]
struct EthGetLogsRequest {
	#[serde(serialize_with = "ser_address_to_hex")]
	address: Address,

	#[serde(serialize_with = "ser_u32_to_hex")]
	from_block: u32,

	#[serde(serialize_with = "ser_u32_to_hex")]
	to_block: u32,
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub struct TxLog {
	#[serde(deserialize_with = "de_hex_to_address")]
	pub(crate) address: Address,

	#[serde(rename = "blockHash", deserialize_with = "de_hex_to_hash")]
	block_hash: H256,

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
	transaction_hash: H256,

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

#[serde(crate = "alt_serde")]
#[derive(Deserialize)]
pub(crate) struct EthGetLogsResponse {
	#[serde(deserialize_with = "de_string_to_bytes")]
	jsonrpc: Vec<u8>,

	id: u32,

	result: Vec<TxLog>,
}

// ETHEREUM INTERCONNECTION MODULE

impl<T: Trait> Module<T> {
	// Returns last block of Ethereum network
	pub(crate) fn get_last_eth_block() -> Result<u32, Error<T>> {
		let params: [(); 0] = [];

		let resp_bytes = Self::make_rpc_request("eth_blockNumber", &params)
			.map_err(|e| {
				debug::error!("cant fetch last eth block: {:?}", e);
				<Error<T>>::HttpFetchingError
			})?;

		// Convert bytes into &str
		let resp_str = str::from_utf8(&resp_bytes)
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		debug::info!("Eth last block response: {}", resp_str);
		let response: EthBlockNumberResponse = serde_json::from_str(resp_str).unwrap();
		Ok(response.result)
	}

	pub(crate) fn fetch_events(address: &str, from_block: u32, to_block: u32) -> Result<Vec<TxLog>, Error<T>> {
		let params = EthGetLogsRequest {
			address: Address::from_str(address).expect("Wrong address"),
			from_block,
			to_block,
		};

		debug::info!("Ser:{}", serde_json::to_string(&params).unwrap());

		let resp_bytes = Self::make_rpc_request("eth_getLogs", &[params])
			.map_err(|e| {
				debug::error!("cant fetch logs from {} to {} block: {:?}", from_block, to_block, e);
				<Error<T>>::HttpFetchingError
			})?;

		// Convert bytes into &str
		let resp_str = str::from_utf8(&resp_bytes)
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		debug::info!("Eth logs response: {}", resp_str);
		let response: EthGetLogsResponse = serde_json::from_str(resp_str).unwrap();
		Ok(response.result)
	}

	// Make an rpc request to JSON RPC provider
	fn make_rpc_request<P>(method: &'static str, params: P) -> Result<Vec<u8>, Error<T>>
		where P: Serialize {
		let body = JSONRpcRequest {
			jsonrpc: "2.0",
			method,
			params,
			id: 1,
		};

		let body = serde_json::to_string(&body).expect("Cant marshal");
		let eth_provider_url = T::EthProviderEndpoint::get();

		let body = vec![body];
		let request = rt_offchain::http::Request::post(eth_provider_url, body);

		// Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
		let timeout = sp_io::offchain::timestamp()
			.add(rt_offchain::Duration::from_millis(FETCH_TIMEOUT_PERIOD));

		// For github API request, we also need to specify `user-agent` in http request header.
		// See: https://developer.github.com/v3/#user-agent-required
		let pending = request
			// .add_header("User-Agent", HTTP_HEADER_USER_AGENT)
			.deadline(timeout) // Setting the timeout time
			.send() // Sending the request out by the host
			.map_err(|e| {
				debug::error!("cant make JSON RPC Call: {:?}", e);
				<Error<T>>::HttpFetchingError
			})?;

		// By default, the http request is async from the runtime perspective. So we are asking the
		// runtime to wait here.
		// The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
		// ref: https://substrate.dev/rustdocs/v2.0.0/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
		let response = pending
			.try_wait(timeout)
			.map_err(|_| <Error<T>>::HttpFetchingError)?
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		// Get all bytes from body
		let resp_bytes = response.body().collect::<Vec<u8>>();

		// Next we fully read the response body and collect it to a vector of bytes.
		Ok(resp_bytes)
	}
}
