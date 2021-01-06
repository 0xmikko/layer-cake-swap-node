use sp_std::prelude::*;
use frame_support::{debug};
use ethabi::{Address, Uint, Log, Token, EventParam, ParamType, Event, RawLog};
use sp_std::convert::{TryInto, TryFrom};
use crate::events::ConvertError::{CantConvertFrom, CantConvertTo, CantConvertAmount};
use super::{Error, Module, Trait};
use sp_std::fmt;
use sp_std::fmt::{Formatter, Debug, Display};
use crate::ethjsonrpc::TxLog;
use hex::encode;

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


impl<T: Trait> Module<T> {
	pub(crate) fn getERC20TransferEvents(address: &'static str, from_block: u32, to_block: u32) -> Result<Vec<ERC20Event>, Error<T>> {
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

		let fetched_events = Self::fetch_events(address, from_block, to_block)?;

		debug::info!("Fetched {} events", fetched_events.len());
		let mut result: Vec<ERC20Event> = vec![];

		for log in fetched_events {
			let raw_log = RawLog::from_tx(log);

			match event.parse_log(raw_log) {
				Ok(log) => {
					let erc20event = ERC20Event::try_from(log)
						.map_err(|_| <Error<T>>::EventParsingError)?;
					result.push(erc20event);
				}
				Err(e) => {
					debug::error!("Cant convert log:{}", e);
					continue
				}
			}
		}

		Ok(result)
	}
}
