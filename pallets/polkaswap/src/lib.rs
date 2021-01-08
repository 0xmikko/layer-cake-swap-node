#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{debug, decl_error, decl_event, decl_module, decl_storage, dispatch,
					dispatch::DispatchResult, Hashable, traits::Get};
use frame_system::{
	self as system, ensure_none, ensure_signed,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
		SignedPayload, SigningTypes, Signer, SubmitTransaction,
	},
};
use core::{convert::*, fmt};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	RuntimeDebug,
	offchain as rt_offchain,
	offchain::{
		storage::StorageValueRef,
		storage_lock::{StorageLock, BlockAndTime},
	},
	// transaction_validity::{
	// 	InvalidTransaction, TransactionSource, TransactionValidity,
	// 	ValidTransaction,
	// },
};
use sp_std::{
	prelude::*, str,
	collections::vec_deque::VecDeque,
};
use sp_runtime::offchain::http::Request;
use sp_runtime::offchain::http::Method::Post;
use codec::{Decode, Encode};
use ethereum_types::Address;
use sp_std::str::FromStr;
use crate::methods::ContractMethod;
// use ethereum::{Bytes, Event, EventParam, Hash, Log, ParamType, RawLog};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
mod ethjsonrpc;
mod serde_helpers;
mod events;
mod methods;
mod offchain_tx;

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

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrapper.
/// We can utilize the supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// them with the pallet-specific identifier.
pub mod crypto {
	use crate::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	use sp_runtime::{
		traits::Verify,
		MultiSignature, MultiSigner,
	};

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
	 trait Store for Module<T: Trait> as Simple {
        // The last value passed to `set_value`.
        // Used as an example of a `StorageValue`.
        pub EthLastBlock get(fn last_value): u32;
        // The value each user has put into `set_value`.
        // Used as an example of a `StorageMap`.
        pub UserValue get(fn user_value): map hasher(blake2_128_concat) T::AccountId => u64;
        pub UserToken get(fn user_token): map hasher(blake2_128_concat) Vec<u8> => Vec<u8> ;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	 pub enum Event<T> where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        // An event which is emitted when `set_value` is called.
        // Contains information about the user who called the function
        // and the value they called with.
        ValueSet(AccountId, u32),
        AnonValueSet(u64),
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

		EventParsingError
	}
}

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
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_value(origin, value: u32) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            debug::info!("SetValue");
            EthLastBlock::put(value);

            // UserValue::<T>::insert(&sender, value);
            // UserToken::insert(3, value);
            Self::deposit_event(RawEvent::ValueSet(sender, value));
            Ok(())
        }

        #[weight = 0]
        pub fn set_value_str(origin, value: Vec<u8>) {
        	 let sender = ensure_signed(origin)?;
             UserToken::insert(value.clone(), value.clone());
            // UserValue::<T>::insert(&sender, value);
            // Self::deposit_event(RawEvent:: AnonValueSet(value));
        }

        // Offchain worker runs after each block
		fn offchain_worker(block_number: T::BlockNumber) {
			debug::info!("Entering off-chain worker");
			debug::info!("{}", T::EthProviderEndpoint::get());
			Self::offchain_signed_tx(block_number);
			}

    }
}

