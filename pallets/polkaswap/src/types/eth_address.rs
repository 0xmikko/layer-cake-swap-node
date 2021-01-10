use codec::{Decode, Encode, EncodeLike, Error, Input, Output};
use ethabi::Address;
use frame_support::debug;
use sp_std::{fmt, prelude::*};
use sp_std::fmt::{Display, Formatter};
use sp_std::str::FromStr;
use hex::encode;

/// EthAddress struct
/// a wrapper for Address stuct with Encode, Decode traits
/// implemented for Parity codec
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct EthAddress(Address);

impl Encode for EthAddress {
	fn encode_to<T: Output>(&self, dest: &mut T) {
		self.using_encoded(|buf| dest.write(buf));
	}

	fn encode(&self) -> Vec<u8> {
		Vec::from(self.0.as_bytes())
	}
}

impl EncodeLike for EthAddress {}

impl Decode for EthAddress {
	fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
		let mut sender_bytes: Vec<u8> = vec![0; 20];
		match value.read(&mut sender_bytes) {
			Ok(_) => {
				Ok(EthAddress(Address::from_slice(&*sender_bytes))) }

			Err(e) => {
				debug::error!("cant convert ETH Address: {}", e.what());
				Err(Error::from("Cant decode eth address"))
			}
		}
	}
}

impl Display for EthAddress {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl From<Address> for EthAddress {
	fn from(value: Address) -> Self {
		EthAddress(value)
	}
}

impl FromStr for EthAddress {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match Address::from_str(s) {
			Ok(v) => { Ok(EthAddress::from(v)) }
			Err(_) => { Err(Error::from("Cant convert str to EthAddress")) }
		}
	}
}
