use frame_support::debug;
use codec::{Encode, EncodeLike, Decode, Error, Input, Output};
use core::cmp::{Ord, Ordering};

use ethabi::Uint;
use sp_std::fmt::{Display, Formatter};
use sp_std::{fmt, prelude::*};
use sp_std::ops::{Add, Sub, Div, Mul};
use frame_support::traits::IsType;
use hex::encode;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct Uint256(Uint);

impl Encode for Uint256 {
	fn encode_to<T: Output>(&self, dest: &mut T) {
		self.using_encoded(|buf| dest.write(buf));
	}

	fn encode(&self) -> Vec<u8> {
		let mut res = vec![0; 32];
		self.0.to_little_endian(res.into_mut());
		res
	}
}

impl EncodeLike for Uint256 {}

impl Decode for Uint256 {
	fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
		let mut amount_bytes: Vec<u8> = vec![0; 32];

		match value.read(&mut amount_bytes) {
			Ok(_) => {
				Ok(Uint256(Uint::from_little_endian(amount_bytes.as_slice())))
			}
			Err(e) => {
				debug::error!("cant convert uin256: {:?}", e);
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

impl From<i32> for Uint256 {
	fn from(num: i32) -> Self {
		Uint::from(num).into()
	}
}

impl From<u128> for Uint256 {
	fn from(num: u128) -> Self {
		Uint::from(num).into()
	}
}

impl Add for Uint256 {
	type Output = Uint256;

	fn add(self, rhs: Self) -> Self::Output {
		(rhs.0 + self.0).into()
	}
}

impl Sub for Uint256 {
	type Output = Uint256;

	fn sub(self, rhs: Self) -> Self::Output {
		(self.0 - rhs.0).into()
	}
}

impl Mul for Uint256 {
	type Output = Uint256;

	fn mul(self, rhs: Self) -> Self::Output {
		(self.0 * rhs.0).into()
	}
}

impl Div for Uint256 {
	type Output = Uint256;

	fn div(self, rhs: Self) -> Self::Output {
		(self.0 / rhs.0).into()
	}
}

impl Ord for Uint256 {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.cmp(&other.0)
	}

	fn max(self, other: Self) -> Self where
		Self: Sized, {
		if self.cmp(&other) == Ordering::Greater {
			self
		} else { other }
	}

	fn min(self, other: Self) -> Self where
		Self: Sized, {
		if self.cmp(&other) == Ordering::Less {
			self
		} else { other }
	}

}

impl PartialOrd for Uint256 {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.0.partial_cmp(&other.0)
	}
}
