#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch,
					traits::Get, debug, Hashable};
use frame_system::{ensure_signed, ensure_none};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
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
        pub LastValue get(fn last_value): u64;
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
        ValueSet(AccountId, u64),
        AnonValueSet(u64),
    }
);

// Errors inform users that something went wrong.
// decl_error! {
// 	pub enum Error for Module<T: Trait> {
// 		/// Error names should be descriptive.
// 		NoneValue,
// 		/// Errors should have helpful documentation associated with them.
// 		StorageOverflow,
// 	}
// }

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	 pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // A default function for depositing events in our runtime
        fn deposit_event() = default;

     	/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_value(origin, value: u64) {
            let sender = ensure_signed(origin)?;
            debug::info!("SetValue");
            LastValue::put(value);

            UserValue::<T>::insert(&sender, value);
            // UserToken::insert(3, value);
            Self::deposit_event(RawEvent::ValueSet(sender, value));
        }

        #[weight = 0]
        pub fn set_value_str(origin, value: Vec<u8>) {
        	 let sender = ensure_signed(origin)?;
             UserToken::insert(value.clone(), value.clone());
            // UserValue::<T>::insert(&sender, value);
            // Self::deposit_event(RawEvent:: AnonValueSet(value));
        }
    }
}
