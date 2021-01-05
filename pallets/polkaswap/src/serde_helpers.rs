use alt_serde::{Serialize, Serializer, Deserializer, Deserialize};
use ethereum_types::{Address, U256};
use sp_std::str::{FromStr};
use sp_std::prelude::*;
use frame_support::{debug};
use alt_serde::de::Error;
use hex::encode;



// SERDE HELPERS FOR CONVERTING STRINGS INTO DIFFERENT TYPES


pub fn ser_u32_to_hex<S>(value: &u32, ser: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
	let hexValue =  encode(value.to_be_bytes());
	let result =  ["0x", hexValue.as_str()].concat();

	ser.serialize_str(result.as_str())
}

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
	Ok(u32::from_str_radix(s, 16).expect("Cant convert"))
}

// Convert HEX 0x string into ethereum address
pub fn de_hex_to_address<'de, D>(de: D) -> Result<Address, D::Error>
	where D: Deserializer<'de> {
	let s: &str = Deserialize::deserialize(de)?;
	// Remove prefix 0x
	let addr = Address::from_str(s)
		.map_err(|e| {
			debug::error!("cant make JSON RPC Call: {:?}", e);
			<D as alt_serde::Deserializer<'de>>::Error::custom("rer")
		})?;
	Ok(addr)
}

// Convert value to Uint256
pub fn de_hex_to_uint256<'de, D>(de: D) -> Result<U256, D::Error>
	where D: Deserializer<'de> {
	let s: &str = Deserialize::deserialize(de)?;
	// Remove prefix 0x
	Ok(U256::from_str_radix(s, 16).expect("Cant convert"))
}
