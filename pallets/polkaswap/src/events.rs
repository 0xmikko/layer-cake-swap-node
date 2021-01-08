use ethabi::{Address, Event, EventParam, Hash, Log, ParamType, RawLog, Token, Uint};
use ethereum_types::{H160, H256};
use frame_support::debug;
use hex::{decode, encode};
use sha3::{Digest, Keccak256};
use sp_std::convert::*;
use sp_std::fmt;
use sp_std::fmt::{Debug, Display, Formatter};
use sp_std::hash;
use sp_std::prelude::*;
use sp_std::str::FromStr;
use sp_std::vec;
use core::hash::Hasher;

use crate::{TOKEN_CONTRACT_ADDRESS, VAULT_CONTRACT_ADDRESS};
use crate::ethjsonrpc::TxLog;
use crate::events::ConvertError::{CantConvertAmount, CantConvertFrom, CantConvertTo};

use super::{Error, Module, Trait};
use crate::methods::{ContractMethod, SenderAmount};

pub struct ERC20Event {
	from: Address,
	to: Address,
	amount: Uint,
}

impl Display for ERC20Event {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "(from: {}, to: {}, amount: {})", self.from, self.to, self.amount)
	}
}

pub enum ConvertError {
	CantConvertFrom,
	CantConvertTo,
	CantConvertAmount,
}

impl Debug for ConvertError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			CantConvertFrom => write!(f, "Cant convert from field"),
			CantConvertTo => write!(f, "Cant convert to field"),
			CantConvertAmount => write!(f, "Cant convert amount field")
		}
	}
}

pub trait FromTxLog {
	fn from_tx(tx: TxLog) -> Self;
}

impl FromTxLog for RawLog {
	fn from_tx(tx: TxLog) -> Self {
		RawLog::from((tx.topics, tx.data))
	}
}

impl TryFrom<Log> for ERC20Event {
	type Error = ConvertError;

	fn try_from(value: Log) -> Result<ERC20Event, Self::Error> {
		debug::info!("Try address");
		let from = match value.params[0].value {
			Token::Address(addr) => addr,
			_ => return Err(CantConvertFrom)
		};

		debug::info!("Try to");
		let to = match value.params[1].value {
			Token::Address(addr) => addr,
			_ => return Err(CantConvertTo)
		};

		debug::info!("Try amount");
		let amount = match value.params[2].value {
			Token::Uint(v) => v,
			_ => return Err(CantConvertAmount)
		};


		debug::info!("Done!");
		Ok(ERC20Event { from, to, amount })
	}
}


impl TryFrom<Log> for SenderAmount {
	type Error = ConvertError;

	fn try_from(value: Log) -> Result<SenderAmount, Self::Error> {
		let sender = match value.params[0].value {
			Token::Address(addr) => addr,
			_ => return Err(CantConvertFrom)
		};

		let amount = match value.params[2].value {
			Token::Uint(v) => v,
			_ => return Err(CantConvertAmount)
		};

		Ok(SenderAmount { sender, amount })
	}
}

fn get_topic_hash(topic: &[u8]) -> Hash {
	Hash::from_slice(&*Keccak256::digest(topic)).into()
}

impl<T: Trait> Module<T> {
	pub(crate) fn get_erc20transfer_events(address: &'static str, from_block: u32, to_block: u32) -> Result<Vec<ContractMethod>, Error<T>> {
		let fetched_events = Self::fetch_events(address, from_block, to_block)?;

		debug::info!("Fetched {} events", fetched_events.len());
		let mut result: Vec<ContractMethod> = vec![];

		// ERC-20 TRANSFER TOPIC
		let erc_20_transfer_topic = get_topic_hash(b"Transfer(address,address,uin256)");

		// VAULT CONTRACT ADDRESS CONVERSATION
		let vault_contract_address = Address::from_str(VAULT_CONTRACT_ADDRESS)
			.map_err(|_| <Error<T>>::EventParsingError)?;

		// TOKEN CONTRACT ADDRESS CONVERSATION
		let token_contract_address = Address::from_str(TOKEN_CONTRACT_ADDRESS)
			.map_err(|_| <Error<T>>::EventParsingError)?;


		for tx_log in fetched_events {
			if tx_log.topics.len() == 0 {
				continue;
			}

			let topic = tx_log.topics[0].clone();
			let address = tx_log.address.clone();
			let raw_log = RawLog::from_tx(tx_log);

			match address {
				vault_contract_address => {
					debug::info!("parsing vault event");
					if let Some(cmd) = Self::parse_vault_transfer_event(&topic, raw_log) {
						result.push(cmd);
					}
				}
				token_contract_address => {
					debug::info!("parsing token contract event");
					if let Some(cmd) = Self::parse_token_transfer_event(raw_log, &vault_contract_address) {
						result.push(cmd);
					}
				}
				(_) => { debug::info!("Nothing to parse") }
			}
		}

		Ok(result)
	}

	fn parse_token_transfer_event(raw_log: RawLog, vault_address: &Address) -> Option<ContractMethod> {
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
			}];

		let event = Event {
			name: "Transfer".into(),
			inputs: params,
			anonymous: false,
		};

		match event.parse_log(raw_log) {
			Ok(log) => {
				if let Ok(erc20event) = ERC20Event::try_from(log) {
					if erc20event.to == *vault_address {
						Some(ContractMethod::DepositToken(SenderAmount { sender: erc20event.from, amount: erc20event.amount }))
					} else { None }
				} else { None }
			}
			Err(e) => {
				debug::error!("Cant parse erc20 transfer event:{}", e);
				None
			}
		}
	}

	fn parse_vault_transfer_event(topic: &Hash, raw_log: RawLog) -> Option<ContractMethod> {
		let deposit_eth_topic = get_topic_hash(b"DepositETH(address,uin256)");
		let withdraw_topic = get_topic_hash(b"Withdraw(address,uin256)");
		let swap_to_token_topic = get_topic_hash(b"SwapToToken(address,uin256)");
		let swap_to_eth_topic = get_topic_hash(b"SwapToToETH(address,uin256)");
		let add_liquidity_topic = get_topic_hash(b"AddLiquidity(address,uin256)");
		match *topic {
			deposit_eth_topic => {
				if let Some(sa) = parse_sender_value_event("DepositETH", raw_log) {
					Some(ContractMethod::DepositETH(sa))
				} else { None }
			}
			withdraw_topic => {
				if let Some(sa) = parse_sender_value_event("Withdraw", raw_log) {
					Some(ContractMethod::Withdraw(sa))
				} else { None }
			}

			swap_to_token_topic => {
				if let Some(sa) = parse_sender_value_event("SwapToToken", raw_log) {
					Some(ContractMethod::SwapToToken(sa))
				} else { None }
			}

			swap_to_eth_topic => {
				if let Some(sa) = parse_sender_value_event("SwapToETH", raw_log) {
					Some(ContractMethod::SwapToETH(sa))
				} else { None }
			}
		}
	}
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
		}];

	let event = Event {
		name: event.into(),
		inputs: params,
		anonymous: false,
	};

	match event.parse_log(raw_log) {
		Ok(log) => {
			match SenderAmount::try_from(log) {
				Ok(sa) => Some(sa),
				Err(_) => None
			}
		}
		Err(_) => None
	}
}
