use ethabi::{Log, Token};
use core::{ convert::*, fmt::Debug};
use codec::{Encode, Decode};
use sp_std::{
	prelude::*,
};

use crate::errors::{ConvertError, ConvertError::*};
use crate::types::{EthAddress, Uint256};

#[derive(Debug, Encode, Decode, Eq, PartialEq, Copy, Clone)]
pub struct SenderAmount {
	pub sender: EthAddress,
	pub amount: Uint256,
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

		Ok(SenderAmount { sender: sender.into(), amount: amount.into() })
	}
}
