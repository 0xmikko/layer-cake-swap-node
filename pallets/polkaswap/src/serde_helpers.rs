use alt_serde::{Serializer, Deserializer, Deserialize};
use ethereum_types::{Address, U256, H256};
use sp_std::str::{FromStr};
use sp_std::prelude::*;
use frame_support::{debug};
use alt_serde::de::{Error, Visitor, SeqAccess};
use hex::{encode, decode};
use ethabi::Hash;
use sp_std::fmt;
use sp_std::fmt::Formatter;

// SERDE HELPERS FOR CONVERTING STRINGS INTO DIFFERENT TYPES

pub(crate) type EthHash = [u8; 32];

// SERIALIZERS

pub fn ser_u32_to_hex<S>(value: &u32, ser: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
	let hex_value = encode(value.to_be_bytes());
	let result = ["0x", hex_value.as_str()].concat();
	ser.serialize_str(result.as_str())
}

pub fn ser_address_to_hex<S>(value: &Address, ser: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
	let hex_value = encode(value.as_bytes());
	let result = ["0x", hex_value.as_str()].concat();
	ser.serialize_str(result.as_str())
}

// DESERIALIZERS


// Convert string into bytes
pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
	where D: Deserializer<'de> {
	let s: &str = Deserialize::deserialize(de)?;
	Ok(s.as_bytes().to_vec())
}

// Convert HEX 0x string into u32
pub fn de_hex_to_u32<'de, D>(de: D) -> Result<u32, D::Error>
	where D: Deserializer<'de> {
	let s0x: &str = Deserialize::deserialize(de)?;
	// Remove prefix 0x
	let s = &s0x[2..];
	let result = u32::from_str_radix(s, 16)
		.map_err(|e| {
			debug::error!("cant deserialize u32: {:?}", e);
			<D as alt_serde::Deserializer<'de>>::Error::custom("Can deserialize u32")
		})?;
	Ok(result)
}

// Convert HEX 0x string into ethereum address
pub fn de_hex_to_address<'de, D>(de: D) -> Result<Address, D::Error>
	where D: Deserializer<'de> {
	let s0x: &str = Deserialize::deserialize(de)?;
	// Remove prefix 0x
	let s = &s0x[2..];
	let addr = Address::from_str(s)
		.map_err(|e| {
			debug::error!("cant deserialize address: {:?}", e);
			<D as alt_serde::Deserializer<'de>>::Error::custom("Can deserialize address")
		})?;
	Ok(addr)
}


// Convert HEX 0x string into ethereum hash(H256)
pub fn de_hex_to_hash<'de, D>(de: D) -> Result<Hash, D::Error>
	where D: Deserializer<'de> {
	let s0x: &str = Deserialize::deserialize(de)?;
	// Remove prefix 0x
	let s = &s0x[2..];
	let hash = Hash::from_str(s)
		.map_err(|e| {
			debug::error!("cant deserialize H256: {:?}", e);
			<D as alt_serde::Deserializer<'de>>::Error::custom("Can deserialize hash")
		})?;
	Ok(hash)
}

// Convert value to Uint256
pub fn de_hex_to_uint256<'de, D>(de: D) -> Result<U256, D::Error>
	where D: Deserializer<'de> {
	let s: &str = Deserialize::deserialize(de)?;
	// Remove prefix 0x
	Ok(U256::from_str_radix(s, 16).expect("Cant convert"))
}

// Convert data in Hex to Vec<u8>
pub fn de_hex_to_vec_u8<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
	where D: Deserializer<'de> {
	let s0x: &str = Deserialize::deserialize(de)?;
	// Remove prefix 0x
	let s = &s0x[2..];
	let result = decode(s)
		.map_err(|e| {
			debug::error!("cant deserialize u32: {:?}", e);
			<D as alt_serde::Deserializer<'de>>::Error::custom("Can deserialize u32")
		})?;
	Ok(result)
}

// Convert Sequence of hash256 to Vec<Hash>
pub fn decode_hex_hash_seq<'de, D>(deserializer: D) -> Result<Vec<Hash>, D::Error>
	where D: Deserializer<'de>
{
	struct HashVec;

	impl<'de> Visitor<'de> for HashVec {
		type Value = Vec<&'de str>;

		fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
			formatter.write_str("hex string")
		}

		fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
			where
				S: SeqAccess<'de>,
		{
			let mut result: Vec<&'de str> = vec![];
			while let Some(value) = seq.next_element()? {
				result.push(value)
			}

			Ok(result)
		}
	}

	let str_result = deserializer.deserialize_seq(HashVec)?;


	let mut result: Vec<Hash> = vec![];
	for hex_str in str_result {
		let s = &hex_str[2..];

		let hash = Hash::from_str(s)
			.map_err(|e| {
				debug::error!("cant deserialize H256: {:?}", e);
				<D as alt_serde::Deserializer<'de>>::Error::custom("Can deserialize hash")
			})?;

		result.push(hash);
	}
	Ok(result)
}
