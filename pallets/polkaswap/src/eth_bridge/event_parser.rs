use core::convert::*;

use ethabi::{Address, Event, EventParam, Hash, ParamType, RawLog};
use frame_support::debug;
use sha3::{Digest, Keccak256};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;
use sp_std::str::FromStr;

use crate::{TOKEN_CONTRACT_ADDRESS, VAULT_CONTRACT_ADDRESS};
use crate::{Error, Module, Trait};
use crate::entities::{BlockEvents, ContractMethod, EthAddress, SenderAmount, Uint256};
use crate::eth_bridge::vault::EventVaultParser;

use super::payloads::{ERC20Event, FromTxLog};

// ERC20 TOKEN TRANSFER
const EVENT_ERC20_TRANSFER: &[u8] = b"Transfer(address,address,uint256)";

impl<T: Trait> Module<T> {
	pub(crate) fn get_block_events(block_number: u32) -> Result<BlockEvents, Error<T>> {
		let fetched_events = Self::fetch_events(block_number)?;

		debug::info!("Fetched {} events", fetched_events.len());

		let mut result: Vec<ContractMethod> = vec![];

		let vault_parser = EventVaultParser::new();

		// VAULT CONTRACT ADDRESS CONVERSATION
		let vault_contract_address = Address::from_str(VAULT_CONTRACT_ADDRESS)
			.map_err(|_| <Error<T>>::EventParsingError)
			.unwrap();

		// TOKEN CONTRACT ADDRESS CONVERSATION
		let token_contract_address = Address::from_str(TOKEN_CONTRACT_ADDRESS)
			.map_err(|_| <Error<T>>::EventParsingError)
			.unwrap();

		debug::info!("Got {} events:", fetched_events.len());
		for tx_log in &fetched_events {
			if tx_log.topics.len() == 0 {
				continue;
			}

			debug::info!("{:?}", tx_log.topics);

			let topic = tx_log.topics[0].clone();
			let address = tx_log.address.clone();
			let raw_log = RawLog::from_tx(tx_log);

			if address == vault_contract_address {
				debug::info!("parsing vault event");
				if let Some(cmd) = vault_parser.parse(&topic, raw_log) {
					debug::info!("Parsed event: {:?}", cmd);
					result.push(cmd);
				}
			} else if address == token_contract_address
				&& topic == get_topic_hash(EVENT_ERC20_TRANSFER)
			{
				debug::info!("parsing token contract event");
				if let Some(cmd) =
				parse_token_transfer_event(raw_log, &vault_contract_address)
				{
					result.push(cmd);
				}
			};
		}

		Ok(BlockEvents {
			block_number,
			methods: result,
		})
	}
}

fn parse_token_transfer_event(
	raw_log: RawLog,
	vault_address: &Address,
) -> Option<ContractMethod> {
	let event = Event {
		name: "Transfer".into(),
		inputs: vec![
			EventParam {
				name: "from".into(),
				kind: ParamType::Address,
				indexed: true,
			},
			EventParam {
				name: "to".into(),
				kind: ParamType::Address,
				indexed: true,
			},
			EventParam {
				name: "value".into(),
				kind: ParamType::Uint(256),
				indexed: false,
			},
		],
		anonymous: false,
	};

	match event.parse_log(raw_log.clone()) {
		Ok(log) => {
			if let Ok(erc20event) = ERC20Event::try_from(log) {
				if erc20event.to == *vault_address {
					Some(ContractMethod::DepositToken(SenderAmount {
						sender: erc20event.from.into(),
						amount: erc20event.amount.into(),
					}))
				} else {
					None
				}
			} else {
				None
			}
		}
		Err(e) => {
			debug::error!("Cant parse erc20 transfer event:{}", e);
			debug::error!("{:?}", raw_log);
			None
		}
	}
}

pub fn get_topic_hash(topic: &[u8]) -> Hash {
	Hash::from_slice(&*Keccak256::digest(topic)).into()
}


