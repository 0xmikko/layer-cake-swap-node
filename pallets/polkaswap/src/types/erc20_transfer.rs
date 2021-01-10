use ethabi::{Address, Uint, Log, Token};
use frame_support::debug;
use core::fmt::Display;
use core::{fmt};
use crate::errors::{ConvertError, ConvertError::*};
use sp_std::convert::TryFrom;

pub struct ERC20Event {
	pub from: Address,
	pub to: Address,
	pub amount: Uint,
}

impl Display for ERC20Event {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "(from: {}, to: {}, amount: {})", self.from, self.to, self.amount)
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
