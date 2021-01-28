use core::convert::*;

use ethabi::{Address, Event, EventParam, Hash, ParamType, RawLog};
use frame_support::debug;
use sha3::{Digest, Keccak256};
use sp_std::prelude::*;
use sp_std::str::FromStr;

use super::payloads::{ERC20Event, FromTxLog};
use crate::entities::{BlockEvents, ContractMethod, EthAddress, SenderAmount, Uint256};
use crate::{TOKEN_CONTRACT_ADDRESS, VAULT_CONTRACT_ADDRESS};

use crate::{Error, Module, Trait};

impl<T: Trait> Module<T> {
	pub(crate) fn get_block_events(block_number: u32) -> Result<BlockEvents, Error<T>> {
		let fetched_events = Self::fetch_events(block_number)?;

		debug::info!("Fetched {} events", fetched_events.len());

		let mut result: Vec<ContractMethod> = vec![];

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
				if let Some(cmd) = Self::parse_vault_transfer_event(&topic, raw_log) {
					debug::info!("Parsed event: {:?}", cmd);
					result.push(cmd);
				}
			} else if address == token_contract_address
				&& topic == get_topic_hash(b"Transfer(address,address,uint256)")
			{
				debug::info!("parsing token contract event");
				if let Some(cmd) =
					Self::parse_token_transfer_event(raw_log, &vault_contract_address)
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

	fn parse_token_transfer_event(
		raw_log: RawLog,
		vault_address: &Address,
	) -> Option<ContractMethod> {
		let params = vec![
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
		];

		let event = Event {
			name: "Transfer".into(),
			inputs: params,
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

	fn parse_vault_transfer_event(topic: &Hash, raw_log: RawLog) -> Option<ContractMethod> {
		// let add_liquidity_topic = get_topic_hash(b"AddLiquidity(address,uin256)");

		if *topic == get_topic_hash(b"DepositETH(address,uint256)") {
			if let Some(sa) = parse_sender_value_event("DepositETH", raw_log) {
				Some(ContractMethod::DepositETH(sa))
			} else {
				None
			}
		} else if *topic == get_topic_hash(b"WithdrawToken(address,uint256)") {
			if let Some(sa) = parse_sender_value_event("Withdraw", raw_log) {
				Some(ContractMethod::WithdrawToken(sa))
			} else {
				None
			}
		} else if *topic == get_topic_hash(b"SwapToToken(address,uint256)") {
			if let Some(sa) = parse_sender_value_event("SwapToToken", raw_log) {
				Some(ContractMethod::SwapToToken(sa))
			} else {
				None
			}
		} else if *topic == get_topic_hash(b"SwapToToETH(address,uint256)") {
			if let Some(sa) = parse_sender_value_event("SwapToETH", raw_log) {
				Some(ContractMethod::SwapToETH(sa))
			} else {
				None
			}
		} else {
			debug::info!("EventNotFound {:?}", *topic);
			None
		}
	}
}

fn get_topic_hash(topic: &[u8]) -> Hash {
	Hash::from_slice(&*Keccak256::digest(topic)).into()
}

fn parse_sender_value_event(event: &str, raw_log: RawLog) -> Option<SenderAmount> {
	let params = vec![
		EventParam {
			name: "sender".into(),
			kind: ParamType::Address,
			indexed: true,
		},
		EventParam {
			name: "value".into(),
			kind: ParamType::Uint(256),
			indexed: false,
		},
	];

	let event = Event {
		name: event.into(),
		inputs: params,
		anonymous: false,
	};

	match event.parse_log(raw_log) {
		Ok(log) => {
			if let Ok(sa) = SenderAmount::try_from(log) {
				Some(sa)
			} else {
				debug::error!("parse_sender_value_event");
				None
			}
		}
		Err(_) => {
			debug::error!("parse_sender_value_event");
			None
		}
	}
}
