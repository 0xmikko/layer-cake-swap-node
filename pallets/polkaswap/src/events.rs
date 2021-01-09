use ethabi::{Address, Event, EventParam, Hash, Log, ParamType, RawLog, Token, Uint};
use frame_support::debug;
use sha3::{Digest, Keccak256};
use sp_std::convert::*;
use sp_std::fmt;
use sp_std::fmt::{Debug, Display, Formatter};
use sp_std::prelude::*;
use sp_std::str::FromStr;
use sp_std::vec;

use crate::{TOKEN_CONTRACT_ADDRESS, VAULT_CONTRACT_ADDRESS};
use crate::ethjsonrpc::TxLog;
use crate::events::ConvertError::{CantConvertAmount, CantConvertFrom, CantConvertTo};
use crate::methods::{ContractMethod, SenderAmount};

use super::{Error, Module, Trait};
use crate::methods::ContractMethod::DepositToken;
use crate::types::{EthAddress, Uint256, Hash256};
use sp_std::ptr::hash;

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
	fn from_tx(tx: &TxLog) -> Self;
}

impl FromTxLog for RawLog {
	fn from_tx(tx: &TxLog) -> Self {
		RawLog::from((tx.topics.clone(), tx.data.clone()))
	}
}

impl TryFrom<Log> for ERC20Event {
	type Error = ConvertError;

	fn try_from(value: Log) -> Result<ERC20Event, Self::Error> {
		let from = match value.params[0].value {
			Token::Address(addr) => addr,
			_ => return Err(CantConvertFrom)
		};

		let to = match value.params[1].value {
			Token::Address(addr) => addr,
			_ => return Err(CantConvertTo)
		};

		let amount = match value.params[2].value {
			Token::Uint(v) => v,
			_ => return Err(CantConvertAmount)
		};

		Ok(ERC20Event { from, to, amount })
	}
}


impl SenderAmount {
	fn from(value: Log, txHash: Hash256) -> Result<SenderAmount, ConvertError> {
		let sender = match value.params[0].value {
			Token::Address(addr) => addr,
			_ => return Err(CantConvertFrom)
		};

		let amount = match value.params[2].value {
			Token::Uint(v) => v,
			_ => return Err(CantConvertAmount)
		};

		Ok(SenderAmount { tx_hash: txHash, sender: sender.into(), amount: amount.into() })
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



		for tx_log in &fetched_events {
			let tx_hash = tx_log.transaction_hash;

			if tx_log.topics.len() == 0 {
				continue;
			}

			let topic = tx_log.topics[0].clone();
			let address = tx_log.address.clone();
			let raw_log = RawLog::from_tx(&tx_log);

			match address {
				vault_contract_address => {
					// debug::info!("parsing vault event");
					if let Some(cmd) = Self::parse_vault_transfer_event(&topic, raw_log, &tx_hash) {
						result.push(cmd);
					}
				}
				token_contract_address => {
					// debug::info!("parsing token contract event");
					if let Some(cmd) = Self::parse_token_transfer_event(raw_log, &vault_contract_address, &tx_hash) {
						result.push(cmd);
					}
				}
				_ => { debug::info!("Nothing to parse") }
			}
		}

		// Test event

		result.push(DepositToken(SenderAmount {
			tx_hash: fetched_events[0].transaction_hash.into(),
			sender: Address::from_str(TOKEN_CONTRACT_ADDRESS).expect("d").into(),
			amount: Uint::from(10).into(),
		}));

		Ok(result)
	}

	fn parse_token_transfer_event(raw_log: RawLog, vault_address: &Address, tx_hash: &Hash) -> Option<ContractMethod> {
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
						Some(ContractMethod::DepositToken(SenderAmount {
							tx_hash: Hash256::from(*tx_hash),
							sender: erc20event.from.into(),
							amount: erc20event.amount.into(),
						}))
					} else { None }
				} else { None }
			}
			Err(e) => {
				debug::error!("Cant parse erc20 transfer event:{}", e);
				None
			}
		}
	}

	fn parse_vault_transfer_event(topic: &Hash, raw_log: RawLog, tx_hash: &Hash) -> Option<ContractMethod> {
		let deposit_eth_topic = get_topic_hash(b"DepositETH(address,uin256)");
		let withdraw_topic = get_topic_hash(b"Withdraw(address,uin256)");
		let swap_to_token_topic = get_topic_hash(b"SwapToToken(address,uin256)");
		let swap_to_eth_topic = get_topic_hash(b"SwapToToETH(address,uin256)");
		let add_liquidity_topic = get_topic_hash(b"AddLiquidity(address,uin256)");
		match *topic {
			deposit_eth_topic => {
				if let Some(sa) = parse_sender_value_event("DepositETH", raw_log, *tx_hash) {
					Some(ContractMethod::DepositETH(sa))
				} else { None }
			}
			withdraw_topic => {
				if let Some(sa) = parse_sender_value_event("Withdraw", raw_log, *tx_hash) {
					Some(ContractMethod::Withdraw(sa))
				} else { None }
			}

			swap_to_token_topic => {
				if let Some(sa) = parse_sender_value_event("SwapToToken", raw_log, *tx_hash) {
					Some(ContractMethod::SwapToToken(sa))
				} else { None }
			}

			swap_to_eth_topic => {
				if let Some(sa) = parse_sender_value_event("SwapToETH", raw_log, *tx_hash) {
					Some(ContractMethod::SwapToETH(sa))
				} else { None }
			}
		}
	}
}

fn parse_sender_value_event(event: &str, raw_log: RawLog, txHash: Hash) -> Option<SenderAmount> {
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
			match SenderAmount::from(log, txHash.into()) {
				Ok(sa) => Some(sa),
				Err(_) => None
			}
		}
		Err(_) => None
	}
}
