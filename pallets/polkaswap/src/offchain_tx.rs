use frame_system::offchain::{Signer, SendSignedTransaction};
use frame_support::debug;

use super::{Error, Module, Trait, Call};
use sp_runtime::offchain::storage::StorageValueRef;
use crate::storage::LocalStorage;

const SYNC_DELAY : u32 = 3;
const LS_LAST_BLOCK_KEY: &[u8] = b"offchain-polkaswap::storage";

impl<T: Trait> Module<T> {
	/// Offchain Eth Sync method get the latest info from Ethereum and send tx on-chain
	pub fn offchain_eth_sync(_block_number: T::BlockNumber) -> Result<(), Error<T>> {

		// Getting the number of last block from ethereum network
		let last_block_eth = Self::get_last_eth_block()?;

		// Getting last saved blocknumber in local storage
		// If there is no info, we set it as the closest block which should be updated
		let last_block_saved = if let Some(lbs) = Self::storage_get_last_block() {
			lbs
		} else { last_block_eth - SYNC_DELAY -1 };

		// Check that there is blocks which are needed to sync. SYNC_DELAY is needed to
		// set up minimal confirmations. We assume that there is no changes in Ethereum
		// after SYNC_DELAY blocks
		if last_block_saved >= last_block_eth - SYNC_DELAY {
			return Ok(());
		}

		// Set current block to next one we need to update
		let current_block = last_block_saved + 1;

		// Getting block events from ethereum network
		let block_events = Self::get_block_events(current_block)?;
		debug::info!("{:?}", &block_events);


		// Sign transaction with getting info
		let signer = Signer::<T, T::AuthorityId>::any_account();
		let result = signer.send_signed_transaction(|_acct|
			// This is the on-chain function
			Call::sync_eth_block(block_events.clone())
		);

		// Display error if the signed tx fails.
		if let Some((acc, res)) = result {
			if res.is_err() {
				debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
				return Err(<Error<T>>::OffchainSignedTxError);
			}
			// Transaction is sent successfully
			debug::info!("Transaction sent!");
			Self::storage_set_last_block(current_block);
			return Ok(());
		}

		// The case of `None`: no account is available for sending
		debug::error!("No local account available");
		Err(<Error<T>>::NoLocalAcctForSigning)
	}

	pub fn storage_get_last_block() -> Option<u32> {
		// Create a reference to Local Storage value.
		// Since the local storage is common for all offchain workers, it's a good practice
		// to prepend our entry with the pallet name.
		let s_info = StorageValueRef::persistent(LS_LAST_BLOCK_KEY);

		// Local storage is persisted and shared between runs of the offchain workers,
		// offchain workers may run concurrently. We can use the `mutate` function to
		// write a storage entry in an atomic fashion.
		//
		// With a similar API as `StorageValue` with the variables `get`, `set`, `mutate`.
		// We will likely want to use `mutate` to access
		// the storage comprehensively.
		//
		// Ref: https://substrate.dev/rustdocs/v2.0.0/sp_runtime/offchain/storage/struct.StorageValueRef.html
		if let Some(Some(ls)) = s_info.get::<u32>() {
			// gh-info has already been fetched. Return early.
			debug::info!("last block stored: {:?}", ls);
			Some(ls)
		} else {
			None
		}
	}

	pub fn storage_set_last_block(block_num: u32) {
		let s_info = StorageValueRef::persistent(LS_LAST_BLOCK_KEY);
		s_info.set(&block_num);
	}
}
