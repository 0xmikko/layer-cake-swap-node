use ethabi::{Address, Uint};
use sp_std::fmt::{Display, Formatter, Debug};
use sp_std::{fmt, vec};
use codec::{Encode, Decode, Error, Input, Output};
use core::convert::*;
use sp_std::{
	prelude::*, str,
};
use hex::encode;
use frame_support::debug;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ContractMethod {
	DepositToken(SenderAmount),
	DepositETH(SenderAmount),
	Withdraw(SenderAmount),
	SwapToToken(SenderAmount),
	SwapToETH(SenderAmount),
	AddLiquidity(SenderAmount),
	WithdrawLiquidity,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct SenderAmount {
	pub sender: Address,
	pub amount: Uint,
}

impl Encode for SenderAmount {
	// fn size_hint(&self) -> usize {
	// 	52
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

// impl Debug for SenderAmount {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
// 		write!(f, "Sender Amount [sender: {}, amount: {}]", dm.sender, dm.amount)
// 	}
// }

impl Display for ContractMethod {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			ContractMethod::DepositToken(dm) => {
				write!(f, "[Deposit Token]: from: {}, amount: {}", dm.sender, dm.amount)
			}

			ContractMethod::DepositETH(dm) => {
				write!(f, "[Deposit ETH]: from: {}, amount: {}", dm.sender, dm.amount)
			}

			ContractMethod::Withdraw(dm) => {
				write!(f, "[Withdraw]: from: {}, amount: {}", dm.sender, dm.amount)
			}

			ContractMethod::SwapToToken(dm) => {
				write!(f, "[Swap to token]: from: {}, amount: {}", dm.sender, dm.amount)
			}

			ContractMethod::SwapToETH(dm) => {
				write!(f, "[Swap to ETH]: from: {}, amount: {}", dm.sender, dm.amount)
			}

			ContractMethod::AddLiquidity(dm) => {
				write!(f, "[Add liquidity]: from: {}, amount: {}", dm.sender, dm.amount)
			}

			ContractMethod::WithdrawLiquidity => {
				write!(f, "Withdraw liquidity")
			}
		}
	}
}

