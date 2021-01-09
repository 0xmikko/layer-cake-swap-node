use frame_support::debug;
use codec::{Encode, EncodeLike, Decode, Error, Input, Output};

use ethabi::{Address, Uint, Hash};
use sp_std::fmt::{Display, Formatter};
use sp_std::{fmt, prelude::*, convert::TryFrom};
use sp_std::ops::Add;

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
		debug::info!("Encode");
		Vec::from(self.0.as_bytes())
	}
}

impl EncodeLike for EthAddress {}

impl Decode for EthAddress {
	fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
		debug::info!("Decode");
		let mut sender_bytes: Vec<u8> = vec![0; 20];
		match value.read(&mut sender_bytes) {
			Ok(_) => {Ok(EthAddress(Address::from_slice(&*sender_bytes)))}

			Err(e) => {
				debug::error!("cant convert uin256: {}", e.what());
				Err(Error::from("Cant decode eth address"))}
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


#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct Uint256(Uint);

impl Encode for Uint256 {
	fn encode_to<T: Output>(&self, dest: &mut T) {
		self.using_encoded(|buf| dest.write(buf));
	}

	fn encode(&self) -> Vec<u8> {
		Vec::from(self.0.0[0].to_be_bytes())
	}
}

impl EncodeLike for Uint256 {}

impl Decode for Uint256 {
	fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
		let mut amount_bytes: Vec<u8> = vec![0; 8];

		value.read(&mut amount_bytes);
		match Uint::try_from(amount_bytes.as_slice()) {
			Ok(value) => Ok( Uint256(value)),
			Err(e) => {
				debug::error!("cant convert uin256: {}", e);
				Err(Error::from("Cant convert uint256"))
			}
		}
	}
}

impl Display for Uint256 {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl From<Uint> for Uint256 {
	fn from(value: Uint) -> Self {
		Uint256(value)
	}
}

impl From<Uint256> for u128 {
	fn from(value: Uint256) -> Self {
		value.0.as_u128()
	}
}

impl Add for Uint256 {
	type Output = Uint256;

	fn add(self, rhs: Self) -> Self::Output {
		(rhs.0 + self.0).into()
	}
}

/// Hash256 struct
/// a wrapper for Hash stuct with Encode, Decode traits
/// implemented for Parity codec
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Hash256(Hash);

impl Encode for Hash256 {
	fn encode_to<T: Output>(&self, dest: &mut T) {
		self.using_encoded(|buf| dest.write(buf));
	}

	fn encode(&self) -> Vec<u8> {
		debug::info!("Encode");
		Vec::from(self.0.as_bytes())
	}
}

impl EncodeLike for Hash256 {}

impl Decode for Hash256 {
	fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
		debug::info!("Decode");
		let mut sender_bytes: Vec<u8> = vec![0; 20];
		match value.read(&mut sender_bytes) {
			Ok(_) => {Ok(Hash256(Hash::from_slice(&*sender_bytes)))}

			Err(e) => {
				debug::error!("cant convert uin256: {}", e.what());
				Err(Error::from("Cant decode eth address"))}
		}
	}
}

impl Display for Hash256 {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl From<Hash> for Hash256 {
	fn from(value: Hash) -> Self {
		Hash256(value)
	}
}
