use ethabi::Uint;
use ethabi::{Address};
use sp_std::fmt::{Display, Formatter};
use sp_std::fmt;

pub enum ContractMethod {
	DepositToken(SenderAmount),
	DepositETH(SenderAmount),
	Withdraw(SenderAmount),
	SwapToToken(SenderAmount),
	SwapToETH(SenderAmount),
	AddLiquidity(SenderAmount),
	WithdrawLiquidity,
}

impl Display for ContractMethod {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			ContractMethod::DepositToken( dm) => {
				write!(f, "[Deposit Token]: from: {}, amount: {}", dm.sender, dm.amount) }

			ContractMethod::DepositETH(dm) => {
				write!(f, "[Deposit ETH]: from: {}, amount: {}", dm.sender, dm.amount) }

			ContractMethod::Withdraw(dm) => {
				write!(f, "[Withdraw]: from: {}, amount: {}", dm.sender, dm.amount) }

			ContractMethod::SwapToToken(dm) => {
				write!(f, "[Swap to token]: from: {}, amount: {}", dm.sender, dm.amount) }

			ContractMethod::SwapToETH(dm) => {
				write!(f, "[Swap to ETH]: from: {}, amount: {}", dm.sender, dm.amount) }

			ContractMethod::AddLiquidity(dm) => {
				write!(f, "[Add liquidity]: from: {}, amount: {}", dm.sender, dm.amount) }

			ContractMethod::WithdrawLiquidity =>{
				write!(f, "Withdraw liquidity") }
		}
	}
}

pub struct SenderAmount {
	pub sender: Address,
	pub amount: Uint,
}
