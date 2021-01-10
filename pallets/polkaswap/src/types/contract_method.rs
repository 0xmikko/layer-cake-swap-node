use sp_std::fmt::{Display, Formatter, Debug};
use sp_std::fmt;
use codec::{Encode, Decode, Error, Input};
use core::convert::*;
use sp_std::{
	prelude::*,
};

use super::sender_amount::SenderAmount;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ContractMethod {
	DepositToken(SenderAmount),
	DepositETH(SenderAmount),
	Withdraw(SenderAmount),
	SwapToToken(SenderAmount),
	SwapToETH(SenderAmount),
	AddLiquidity(SenderAmount),
	WithdrawLiquidity(SenderAmount),
}

impl Encode for ContractMethod {
	fn encode(&self) -> Vec<u8> {
		match self {
			ContractMethod::DepositToken(sa) => { ContractMethod::encode_item(0u8, sa) }
			ContractMethod::DepositETH(sa) => { ContractMethod::encode_item(1u8, sa) }
			ContractMethod::Withdraw(sa) => { ContractMethod::encode_item(2u8, sa) }
			ContractMethod::SwapToToken(sa) => { ContractMethod::encode_item(3u8, sa) }
			ContractMethod::SwapToETH(sa) => { ContractMethod::encode_item(4u8, sa) }
			ContractMethod::AddLiquidity(sa) => { ContractMethod::encode_item(5u8, sa) }
			ContractMethod::WithdrawLiquidity(sa) => { ContractMethod::encode_item(6u8, sa) }
		}
	}
}

impl Decode for ContractMethod {
	fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
		let cm_type = value.read_byte()?;
		let sa = SenderAmount::decode(value)?;
		match cm_type {
			0u8 => Ok(ContractMethod::DepositToken(sa)),
			1u8 => Ok(ContractMethod::DepositETH(sa)),
			2u8 => Ok(ContractMethod::Withdraw(sa)),
			3u8 => Ok(ContractMethod::SwapToToken(sa)),
			4u8 => Ok(ContractMethod::SwapToETH(sa)),
			5u8 => Ok(ContractMethod::AddLiquidity(sa)),
			6u8 => Ok(ContractMethod::WithdrawLiquidity(sa)),
			_ => { Err(Error::from("Unknown contract method type"))}
		}
	}
}

impl ContractMethod {
	fn encode_item(cm_type: u8, sa: &SenderAmount) -> Vec<u8> {
		let mut cm_bytes: Vec<u8> = Vec::from([cm_type]);
		let mut sa_bytes = sa.encode();
		cm_bytes.append(&mut sa_bytes);
		cm_bytes
	}
}



impl Debug for ContractMethod {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", &self)
	}
}

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

			ContractMethod::WithdrawLiquidity(dm) => {
				write!(f, "[Withdraw liquidity]: from: {}, amount: {}", dm.sender, dm.amount)
			}
		}
	}
}

