#![cfg_attr(not(feature = "std"), no_std)]

use core::{cmp, convert::*};

use codec::Encode;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{debug, decl_error, decl_event, decl_module, decl_storage,
					dispatch::{DispatchError, DispatchResult}, traits::Get};
use frame_system::{
	self as system, ensure_signed,
	offchain::{
		AppCrypto, CreateSignedTransaction,
	},
};
use sp_core::crypto::KeyTypeId;
use sp_std::{
	prelude::*, str,
};

use crate::types::{BlockEvents, ContractMethod::*, EthAddress, SenderAmount, Uint256};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
mod offchain_tx;
mod types;
mod errors;
mod payloads;
mod contract;
mod eth_sync;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When an offchain worker is signing transactions it's going to request keys from type
/// `KeyTypeId` via the keystore to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");
pub const NUM_VEC_LEN: usize = 10;
/// The type to sign and send transactions.
pub const UNSIGNED_TXS_PRIORITY: u64 = 100;

pub const FETCH_TIMEOUT_PERIOD: u64 = 3000;
// in milli-seconds
pub const LOCK_TIMEOUT_EXPIRATION: u64 = FETCH_TIMEOUT_PERIOD + 1000;
// in milli-seconds
pub const LOCK_BLOCK_EXPIRATION: u32 = 3; // in block number

/// Vault contract address
pub const VAULT_CONTRACT_ADDRESS: &'static str = "6b175484e89094c44da98b954eedeac495271d0f";

/// Token contract agaist Eth erc20 contract address
/// DAI on our case
pub const TOKEN_CONTRACT_ADDRESS: &'static str = "6b175474e89094c44da98b954eedeac495271d0f";

/// Initial ratio for token to ETH
/// Used at first withdraw
pub const INITIAL_RATIO : u128 = 100u128;

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrapper.
/// We can utilize the supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// them with the pallet-specific identifier.
pub mod crypto {
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		MultiSignature,
		MultiSigner, traits::Verify,
	};
	use sp_runtime::app_crypto::{app_crypto, sr25519};

	use crate::KEY_TYPE;

	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	// implemented for ocw-runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
	for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}


/// This is the pallet's configuration trait
pub trait Trait: system::Trait + CreateSignedTransaction<Call<Self>> {
	/// The identifier type for an offchain worker.
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	/// The overarching dispatch call type.
	type Call: From<Call<Self>>;
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type EthProviderEndpoint: Get<&'static str>;
}



// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	 trait Store for Module<T: Trait> as PolkaSwap {

	 	/// Last block synced with ethereum
        pub EthLastSyncedBlock get(fn eth_last_synced_block): u32;

        /// Token balance for eth user
        pub TokenBalance get(fn token_balance): map hasher(blake2_128_concat) EthAddress => Uint256;

        /// Eth balance for eth user
        pub EthBalance get(fn eth_balance): map hasher(blake2_128_concat) EthAddress => Uint256;

        /// Pool token liquidity
        pub PoolTokenLiquidity get(fn pool_token_liquidity) : Uint256;

		/// Pool eth liquidity
        pub PoolETHLiquidity get(fn pool_eth_liquidity) : Uint256;

    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	 pub enum Event<T> where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
    	// emitted when user deposit tokens on account
        DepositedToken(Vec<u8>, u128),
        DepositedETH(Vec<u8>, u128),
        WithdrawToken(Vec<u8>, u128),
		SwapToToken(Vec<u8>, u128),
		SwapToETH(Vec<u8>, u128),
		AddLiquidity(Vec<u8>, u128),
		WithdrawLiquidity(Vec<u8>, u128),

		EthBlockSynced(u32),
		ValueSet(AccountId, u32),

		// Errors
		WithdrawTokenError(Vec<u8>),
}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		// Error returned when not sure which ocw function to executed
		UnknownOffchainMux,

		// Error returned when making signed transactions in off-chain worker
		NoLocalAcctForSigning,
		OffchainSignedTxError,

		// Error returned when making unsigned transactions in off-chain worker
		OffchainUnsignedTxError,

		// Error returned when making unsigned transactions with signed payloads in off-chain worker
		OffchainUnsignedTxSignedPayloadError,

		// Error returned when fetching github info
		HttpFetchingError,

		EventParsingError,

		ContractTokenError,
	}
}

pub struct ContractError(&'static str);

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	 pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // A default function for depositing events in our runtime
        fn deposit_event() = default;
        // const ethProviderEndpoint: &'static str = T::EthProviderEndpoint::get();

     	/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		// #[weight = 10_000 + T::DbWeight::get().writes(1)]
        // pub fn set_value(origin, value: u32) -> DispatchResult {
        //     let sender = ensure_signed(origin)?;
        //     debug::info!("SetValue");
        //     EthLastBlock::put(value);
		//
        //     // UserValue::<T>::insert(&sender, value);
        //     // UserToken::insert(3, value);
        //     Self::deposit_event(RawEvent::ValueSet(sender, value));
        //     Ok(())
        // }

		/// SYNC ETH_BLOCK EVENTS
        #[weight = 0]
        pub fn sync_eth_block(origin, be: BlockEvents) -> DispatchResult  {
        	debug::info!("{:?}", be);

			///
			/// ==================== ::CONTRACT FUNCTIONS:: ==========================
			/// Deposit token for user
        	fn deposit_token(sa: SenderAmount) -> Result<SenderAmount, ContractError>{
				let updated_balance = if TokenBalance::contains_key(&sa.sender) {
						sa.amount + TokenBalance::get(&sa.sender)
					} else { sa.amount };

				TokenBalance::insert(&sa.sender, &updated_balance);
				PoolTokenLiquidity::put(Uint256::from(20u128));
				Ok(SenderAmount{ sender: sa.sender, amount: updated_balance })
        	}

			/// Deposit Eth for user
			fn deposit_eth(sa: SenderAmount) -> Result<SenderAmount, ContractError> {
				let updated_balance = if EthBalance::contains_key(&sa.sender) {
									sa.amount + EthBalance::get(&sa.sender)
								} else { sa.amount};

				EthBalance::insert(&sa.sender, &updated_balance);
				Ok(SenderAmount{ sender: sa.sender, amount: updated_balance })
			}

			/// Withdraw function
			/// @return SenderAmount with real numbers to be withdrawn, else ContractError
			fn withdraw_token(sa: SenderAmount) -> Result<SenderAmount, ContractError> {
				if TokenBalance::contains_key(&sa.sender) {
					return Err(ContractError("User not found"));
				}
				let amount = get_min_user_token_balance(&sa);
				if amount.clone() > Uint256::from(0)  {
					let updated_balance = TokenBalance::get(&sa.sender) - amount;
					TokenBalance::insert(&sa.sender, &updated_balance);
					Ok(SenderAmount{ sender: sa.sender, amount: updated_balance })
				} else {
					Err(ContractError("Nothing to with"))
				}
			}


			/// ==================== ::CONTRACT HELPERS:: ==========================
			fn get_min_user_token_balance(sa: &SenderAmount) -> Uint256 {
				let user_token_balance = TokenBalance::get(&sa.sender);
				cmp::min(sa.amount, user_token_balance)
			}

			fn get_ratio() -> Uint256 {
				let token_liquidity = PoolTokenLiquidity::get();
				let eth_liquidity = PoolETHLiquidity::get();
				if token_liquidity.clone() == Uint256::from(0) && eth_liquidity.clone() == Uint256::from(0) {
					return INITIAL_RATIO.into();
				}

				token_liquidity / eth_liquidity
			}


        	/// Get block number of incoming message
        	let block_to_sync = be.block_number;

        	/// Compare with last synced block on-chain
        	/// Adding new info only if it's greater, otherwise finish with error
        	if block_to_sync > EthLastSyncedBlock::get() {

        		/// Iterate by all commands in block
				for cmd in be.methods.clone() {
					match cmd {

						// Deposit Token Method
						DepositToken(sa) => {
							match deposit_token(sa) {
								Ok(res) => {
									Self::deposit_event(RawEvent::DepositedToken(res.sender.encode(), res.amount.into()));
								}
								Err(err) => {
									debug::error!("Can deposit event: {}", err.0);
								}
							}
						}

						// Deposit ETH method
						DepositETH(sa) => {
							match deposit_eth(sa) {
								Ok(res) => {
									Self::deposit_event(RawEvent::DepositedETH(res.sender.encode(), res.amount.into()));
								}
								Err(err) => {
									debug::error!("Can deposit event: {}", err.0);
								}
							}
						}

						// Withdraw method
						WithdrawToken(sa) => {
						match withdraw_token(sa) {
								Ok(res) => {
									Self::deposit_event(RawEvent::WithdrawToken(res.sender.encode(), res.amount.into()));
								}
								Err(err) => {
									debug::error!("Can deposit event: {}", err.0);
									Self::deposit_event(RawEvent::WithdrawTokenError(err.0.encode()))
								}
							}
						 }

						// Swap to token method <==================================================
						SwapToToken(sa) => {
							let user_eth_balance = EthBalance::get(&sa.sender);
							let eth_to_swap = cmp::min(sa.amount, user_eth_balance);

						}
						SwapToETH(sa) => {  }
						AddLiquidity(sa) => {
							let user_token_balance = TokenBalance::get(&sa.sender);
							let user_eth_balance = EthBalance::get(&sa.sender);

							let eth_to_swap = cmp::min(sa.amount, user_eth_balance);
						  }
						WithdrawLiquidity(sa) => {  }
					}
				}

				EthLastSyncedBlock::put(block_to_sync);
				Self::deposit_event(RawEvent::EthBlockSynced(block_to_sync));
        		Ok(())
        	} else {
        		Err(DispatchError::Other("This block is already synced!"))
        	}


        }

        // Offchain worker runs after each block
		fn offchain_worker(block_number: T::BlockNumber) {
			Self::offchain_eth_sync(block_number);
			}

    }
}

