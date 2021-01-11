use sp_std::prelude::*;
use sp_std::fmt::{Debug, Formatter};
use codec::{Encode, Decode};
use crate::data::ContractMethod;
use sp_std::fmt;

#[derive(Eq, Encode, Decode, PartialEq, Clone)]
pub struct BlockEvents {
	pub(crate) block_number: u32,
	pub(crate) methods: Vec<ContractMethod>
}

impl Debug for BlockEvents {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "[ BLOCK FROM ETH TO SYNC ]\nBlock number: {}", &self.block_number)?;
		for cmd in self.methods.clone() {
			write!(f, "{}", cmd)?;
		}
		write!(f, "--------")
	}
}
