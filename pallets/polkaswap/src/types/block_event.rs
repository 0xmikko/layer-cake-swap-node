use core::convert::*;
use sp_std::{
	prelude::*,
};
use crate::types::ContractMethod;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct BlockEvents {
	pub(crate) block_number: u32,
	pub(crate) methods: Vec<ContractMethod>
}
