use ethabi::{Address, Event, EventParam, Hash, ParamType, RawLog};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::convert::TryFrom;
use frame_support::debug;
use sp_std::prelude::*;

use crate::entities::{ContractMethod, SenderAmount};
use super::event_parser::get_topic_hash;

pub struct EventVaultParser {
	events_map: BTreeMap<Hash, EventCmd>,
}

impl<'a> EventVaultParser {
	pub fn new() -> Self {
		let mut result = EventVaultParser { events_map: BTreeMap::new() };
		result.add_event("DepositETH", ContractMethod::DepositETH);
		result.add_event("WithdrawETH", ContractMethod::WithdrawETH);
		result.add_event("WithdrawToken", ContractMethod::WithdrawToken);
		result.add_event("SwapToToken", ContractMethod::SwapToToken);
		result.add_event("SwapToETH", ContractMethod::SwapToETH);
		result.add_event("AddLiquidity", ContractMethod::AddLiquidity);
		result.add_event("RemoveLiquidity", ContractMethod::RemoveLiquidity);
		result
	}

	pub fn parse(&self, hash: &Hash, raw_log: RawLog) -> Option<ContractMethod> {
		if let Some(ec) = self.events_map.get(hash) {
			ec.parse(raw_log)
		} else { None }
	}

	fn add_event(&mut self, event_name: &str, method: fn(SenderAmount) -> ContractMethod) {
		self.events_map.insert(get_vault_topic_hash(event_name),
							   EventCmd::new(event_name, method));
	}
}

pub struct EventCmd {
	event: Event,
	method: fn(SenderAmount) -> ContractMethod,
}

impl EventCmd {
	fn new(event_name: &str, method: fn(SenderAmount) -> ContractMethod) -> EventCmd {
		let event = Event {
			name: event_name.into(),
			inputs: vec![
				EventParam {
					name: "sender".into(),
					kind: ParamType::Address,
					indexed: true,
				},
				EventParam {
					name: "value".into(),
					kind: ParamType::Uint(256),
					indexed: false,
				},
			],
			anonymous: false,
		};

		EventCmd { event, method }
	}

	fn parse(&self, raw_log: RawLog) -> Option<ContractMethod> {
		match self.event.parse_log(raw_log) {
			Ok(log) => {
				if let Ok(sa) = SenderAmount::try_from(log) {
					Some((self.method)(sa))
				} else {
					debug::error!("parse_sender_value_event");
					None
				}
			}
			Err(_) => {
				debug::error!("parse_sender_value_event");
				None
			}
		}
	}
}

fn get_vault_topic_hash(event_name: &str) -> Hash {
	let mut topic: Vec<u8> = event_name.into();
	let mut params: Vec<u8> = b"(address,uint256)".to_vec();
	topic.append(&mut params);
	get_topic_hash(&topic)
}
