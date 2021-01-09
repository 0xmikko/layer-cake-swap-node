use ethabi::{Address, Uint, Hash};
use sp_std::fmt::{Display, Formatter, Debug};
use sp_std::{fmt, vec};
use codec::{Encode, Decode, Error, Input, Output};
use core::convert::*;
use sp_std::{
	prelude::*, str,
};
use hex::encode;
use frame_support::debug;
use crate::types::{EthAddress, Uint256, Hash256};


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

#[derive(Debug, Encode, Decode, PartialEq, Copy, Clone)]
pub struct SenderAmount {
	pub tx_hash: Hash256,
	pub sender: EthAddress,
	pub amount: Uint256,
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

