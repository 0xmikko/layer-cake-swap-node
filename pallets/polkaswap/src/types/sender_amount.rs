use frame_support::debug;

use ethabi::{Address, Uint, Log, Token};
use core::{ convert::*, fmt::Debug};
use codec::{Encode, Decode, Error, Input, Output};
use sp_std::{
	prelude::*,
};

use crate::errors::{ConvertError, ConvertError::*};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct SenderAmount {
	pub sender: Address,
	pub amount: Uint,
}

impl Encode for SenderAmount {
	// fn size_hint(&self) -> usize {
	// 	28/usize
	// }

	fn encode_to<T: Output>(&self, dest: &mut T) {
		self.using_encoded(|buf| dest.write(buf));
	}

	fn encode(&self) -> Vec<u8> {
		let mut sender_bytes = Vec::from(self.sender.as_bytes());
		let mut amount_bytes = Vec::from(self.amount.0[0].to_be_bytes());

		sender_bytes.append(&mut amount_bytes);
		sender_bytes
	}
}

impl Decode for SenderAmount {
	fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
		let len = value.remaining_len()?;
		match len {
			Some(l) => debug::info!("L2: {}", l),
			None => debug::info!("L2: None")
		}
		let mut sender_bytes: Vec<u8> = vec![0; 20];
		value.read(&mut sender_bytes);
		let sender = Address::from_slice(&*sender_bytes);

		let mut amount_bytes: Vec<u8> = vec![0; 8];
		value.read(&mut amount_bytes);
		let amount = Uint::try_from(amount_bytes.as_slice()).expect("Shit");
		Ok(SenderAmount { sender, amount })
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
