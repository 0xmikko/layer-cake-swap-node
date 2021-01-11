use frame_support::debug;
use codec::{Encode, EncodeLike, Decode, Error, Input, Output};

use ethabi::Hash;
use sp_std::fmt::{Display, Formatter};
use sp_std::{fmt, prelude::*};

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
		Vec::from(self.0.as_bytes())
	}
}

impl EncodeLike for Hash256 {}

impl Decode for Hash256 {
	fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
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
