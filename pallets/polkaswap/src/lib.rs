#![cfg_attr(not(feature = "std"), no_std)]

use core::{cmp, convert::*};

use codec::Encode;
use frame_support::{debug, decl_error, decl_event, decl_module, decl_storage,
					dispatch::{DispatchError, DispatchResult}, traits::Get};
use frame_system::{
	self as system, ensure_signed,
	offchain::{
		AppCrypto, CreateSignedTransaction,
	},
};
use sp_core::crypto::KeyTypeId;
use sp_runtime::traits::IntegerSquareRoot;
use sp_std::{
	prelude::*, str,
};
use sp_std::collections::btree_map::BTreeMap;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use sp_std::convert::TryFrom;

use crate::entities::{BlockEvents, ContractMethod::*, EthAddress, SenderAmount, Uint256};

mod offchain;
mod entities;
mod errors;
mod eth_bridge;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When an offchain worker is signing transactions it's going to request keys from type
/// `KeyTypeId` via the keystore to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"plsw");

pub const FETCH_TIMEOUT_PERIOD: u64 = 3000;

// in milli-seconds
pub const LOCK_TIMEOUT_EXPIRATION: u64 = FETCH_TIMEOUT_PERIOD + 1000;

// in milli-seconds
pub const LOCK_BLOCK_EXPIRATION: u32 = 3; // in block number

/// Vault contract address
pub const VAULT_CONTRACT_ADDRESS: &'static str = "aC2CEB0C1AFd2e6674c5C016e13C02B1599daF5E";

/// Token contract agaist Eth erc20 contract address
/// DAI on our case
pub const TOKEN_CONTRACT_ADDRESS: &'static str = "07865c6e87b9f70255377e024ace6630c1eaa37f";

/// Initial ratio for token to ETH
/// Used at first withdraw
pub const INITIAL_RATIO: u128 = 1000u128;

pub const MINIMAL_LIQUIDITY: u128 = 1000u128;

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
	 trait Store for Module<T: Trait> as PolkaSwap {

	 	/// Last block synced with ethereum
        pub EthLastSyncedBlock get(fn eth_last_synced_block): u32;

        /// Token balance for eth user
        pub TokenBalance get(fn token_balance): map hasher(blake2_128_concat) EthAddress => Uint256;

        /// Eth balance for eth user
        pub EthBalance get(fn eth_balance): map hasher(blake2_128_concat) EthAddress => Uint256;

         /// Liquidity balance for eth user
        pub LiquidityBalance get(fn liquidity_balance): map hasher(blake2_128_concat) EthAddress => Uint256;

        /// Pool token liquidity
        pub PoolTokenLiquidity get(fn pool_token_liquidity) : Uint256;

		/// Pool eth liquidity
        pub PoolETHLiquidity get(fn pool_eth_liquidity) : Uint256;

        /// Total supply for liquidity tokens
        pub TotalSupply get(fn total_supply) : Uint256;

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
        WithdrawETH(Vec<u8>, u128),
		SwapToToken(Vec<u8>, u128),
		SwapToETH(Vec<u8>, u128),
		AddLiquidity(Vec<u8>, u128),
		RemoveLiquidity(Vec<u8>, u128),

		EthBlockSynced(u32),
		ValueSet(AccountId, u32),

		// Errors
		ContractError(Vec<u8>),
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

		// Error returned when making unsigned transactions with signed eth_bridge.payloads in off-chain worker
		OffchainUnsignedTxSignedPayloadError,

		// Error returned when fetching github info
		HttpFetchingError,

		EventParsingError,

		ContractTokenError,
	}
}

pub struct ContractError(&'static str);

pub enum ContractEvent {
	DepositedToken(Vec<u8>, u128),
	DepositedETH(Vec<u8>, u128),
	WithdrawToken(Vec<u8>, u128),
	WithdrawETH(Vec<u8>, u128),
	SwapToToken(Vec<u8>, u128),
	SwapToETH(Vec<u8>, u128),
	AddLiquidity(Vec<u8>, u128),
	RemoveLiquidity(Vec<u8>, u128),
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	 pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // A default function for depositing events in our runtime
        pub fn deposit_event() = default;

		/// SYNC ETH_BLOCK EVENTS
		/// It gets Block events entities and update state based on method it contains
		/// After updating state, it updates EthLastSyncedBlock, writing the last block number
		/// @returns DispatchResult
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn sync_eth_block(origin, be: BlockEvents) -> DispatchResult  {
        	debug::info!("{:?}", be);

			///
			/// ==================== ::CONTRACT FUNCTIONS:: ==========================
			/// Deposit token for user
        	fn deposit_token(sa: SenderAmount) -> Result<ContractEvent, ContractError>{
				debug::info!("deposit_token: {:?}", sa);
				let updated_balance = if TokenBalance::contains_key(&sa.sender) {
						sa.amount + TokenBalance::get(&sa.sender)
					} else { sa.amount };

				TokenBalance::insert(&sa.sender, &updated_balance);
				Ok(ContractEvent::DepositedToken(sa.sender.encode(), updated_balance.into()).into())

        	}

			/// Deposit Eth for user
			fn deposit_eth(sa: SenderAmount) -> Result<ContractEvent, ContractError>{
				debug::info!("deposit_eth: {:?}", sa);
				let updated_balance = if EthBalance::contains_key(&sa.sender) {
									sa.amount + EthBalance::get(&sa.sender)
								} else { sa.amount};

				EthBalance::insert(&sa.sender, &updated_balance);
				Ok(ContractEvent::DepositedETH(sa.sender.encode(), updated_balance.into()))
			}

			/// Withdraw function
			/// @return SenderAmount with real numbers to be withdrawn, else ContractError
			fn withdraw_token(sa: SenderAmount) -> Result<ContractEvent, ContractError>{
				if TokenBalance::contains_key(&sa.sender) {
					return Err(ContractError("User not found"));
				}
				let amount = get_min_user_token_balance(&sa);
				if amount.clone() > Uint256::from(0)  {
					let updated_balance = TokenBalance::get(&sa.sender) - amount;
					TokenBalance::insert(&sa.sender, &updated_balance);
					Ok(ContractEvent::WithdrawToken(sa.sender.encode(), updated_balance.into()))
				} else {
					Err(ContractError("Nothing to with"))
				}
			}

			/// Withdraw function
			/// @return SenderAmount with real numbers to be withdrawn, else ContractError
			fn withdraw_eth(sa: SenderAmount) -> Result<ContractEvent, ContractError>{
				if EthBalance::contains_key(&sa.sender) {
					return Err(ContractError("User not found"));
				}
				let amount = get_min_user_eth_balance(&sa);
				if amount.clone() > Uint256::from(0)  {
					let updated_balance = EthBalance::get(&sa.sender) - amount;
					EthBalance::insert(&sa.sender, &updated_balance);
					Ok(ContractEvent::WithdrawETH(sa.sender.encode(), updated_balance.into()))
				} else {
					Err(ContractError("Nothing to with"))
				}
			}

			/// SwapToToken
			fn swap_to_token(sa: SenderAmount) -> Result<ContractEvent, ContractError>{
				debug::info!("swap_to_token");
				let desired_token_amount = sa.amount;
				let pool_token_liquidity = PoolTokenLiquidity::get();
				let pool_eth_liquidity = PoolETHLiquidity::get();

				if desired_token_amount > pool_token_liquidity {
					return Err(ContractError("Not enough token liquidity"));
				}

				let amount_eth_to_withdraw = desired_token_amount / get_ratio() * Uint256::from(1000) / Uint256::from(997);
				let amount_eth_user = EthBalance::get(&sa.sender);

				if amount_eth_to_withdraw > amount_eth_user {
					return Err(ContractError("Not enough eth on user account"));
				}

				let amount_token_user = TokenBalance::get(&sa.sender);

				let updated_user_eth_balance = amount_eth_user - amount_eth_to_withdraw;
				let updated_user_token_balance = amount_token_user + desired_token_amount;

				PoolTokenLiquidity::set(pool_token_liquidity - desired_token_amount);
				PoolETHLiquidity::set(pool_eth_liquidity + amount_eth_to_withdraw);

				TokenBalance::insert(&sa.sender, &updated_user_token_balance);
				EthBalance::insert(&sa.sender, &updated_user_eth_balance);

				Ok(ContractEvent::SwapToToken(sa.sender.encode(), updated_user_token_balance.into()))

			}

			/// SwapToToken
			fn swap_to_eth(sa: SenderAmount) -> Result<ContractEvent, ContractError>{
				debug::info!("swap_to_eth");
				let desired_eth_amount = sa.amount;
				let pool_token_liquidity = PoolTokenLiquidity::get();
				let pool_eth_liquidity = PoolETHLiquidity::get();

				if desired_eth_amount > pool_eth_liquidity {
					return Err(ContractError("Not enough eth liquidity"));
				}

				let amount_token_to_withdraw = desired_eth_amount * get_ratio() * Uint256::from(1000) / Uint256::from(997);
				let amount_token_user = TokenBalance::get(&sa.sender);

				if amount_token_to_withdraw > amount_token_user {
					return Err(ContractError("Not enough eth on user account"));
				}

				let amount_eth_user = EthBalance::get(&sa.sender);

				let updated_user_eth_balance = amount_eth_user + desired_eth_amount;
				let updated_user_token_balance = amount_token_user - amount_token_to_withdraw;

				PoolTokenLiquidity::set(pool_token_liquidity + amount_token_to_withdraw);
				PoolETHLiquidity::set(pool_eth_liquidity - desired_eth_amount);

				TokenBalance::insert(&sa.sender, &updated_user_token_balance);
				EthBalance::insert(&sa.sender, &updated_user_eth_balance);
				Ok(ContractEvent::SwapToETH(sa.sender.encode(), updated_user_eth_balance.into()))

			}

			/// AddLiquidity function
			/// Amount means value in eth
			/// @return SenderAmount with real numbers to be withdrawn, else ContractError
			fn add_liquidity(sa: SenderAmount) -> Result<ContractEvent, ContractError>{

				debug::info!("add_liquidity");
				let total_supply = TotalSupply::get();

				debug::info!("total_supply: {:}", &total_supply);
				let mut amount_eth = get_min_user_eth_balance(&sa);
				let amount_token = amount_eth * get_ratio();
				let user_token_balance = TokenBalance::get(&sa.sender);
				let amount_token_upd = cmp::min(amount_token, user_token_balance);

				if amount_token_upd < amount_token {
					amount_eth = amount_token_upd / get_ratio();
				}
				let amount_token = amount_token_upd;

				debug::info!("amount_eth: {:?}", amount_eth);
				debug::info!("amount_token_upd: {:?}", amount_token);

				let mut liquidity : Uint256;
				let pool_token_liquidity = PoolTokenLiquidity::get();
				let pool_eth_liquidity = PoolETHLiquidity::get();

				if total_supply == Uint256::from(0) {
					let initial_value = u128::from(amount_eth).integer_sqrt() * u128::from(amount_token).integer_sqrt();

					if initial_value < MINIMAL_LIQUIDITY {
						return Err(ContractError("Not enough liquidity"));
					}
					liquidity = (initial_value - MINIMAL_LIQUIDITY).into();

					// LiquidityBalance::insert(EthAddre, &updated_balance);
				} else {

					liquidity = cmp::min(amount_eth * total_supply / pool_eth_liquidity, amount_token * total_supply/pool_token_liquidity);
				}

				let mut user_liquidity_balance = LiquidityBalance::get(&sa.sender);
				user_liquidity_balance = user_liquidity_balance + liquidity;


				let updated_token_balance = user_token_balance - amount_token;
				let updated_eth_balance = EthBalance::get(&sa.sender) - amount_eth;

				TokenBalance::insert(&sa.sender, &updated_token_balance);
				EthBalance::insert(&sa.sender, updated_eth_balance);

				// Updating pool liquidity parameters
				LiquidityBalance::insert(&sa.sender, &user_liquidity_balance);
				PoolETHLiquidity::set(pool_eth_liquidity + amount_eth);
				PoolTokenLiquidity::set(pool_token_liquidity + amount_token);
				TotalSupply::set(total_supply + liquidity);
				Ok(ContractEvent::AddLiquidity(sa.sender.encode(), user_liquidity_balance.into()))

			}

			/// RemoveLiquidity function
			/// Amount means value in eth
			/// @return SenderAmount with real numbers to be withdrawn, else ContractError
			fn remove_liquidity(sa: SenderAmount) -> Result<ContractEvent, ContractError>{
				let amount_to_remove =  get_min_user_liquidity_balance(&sa);
				let total_supply = TotalSupply::get();

				if total_supply == Uint256::from(0) {
					return  Err(ContractError("Not enough liquidity in pool"));
				}

				let pool_eth_liquidity = PoolETHLiquidity::get();
				let pool_token_liquidity = PoolTokenLiquidity::get();


				let amount_eth_to_return = pool_eth_liquidity * amount_to_remove / total_supply;
				let amount_token_to_return = pool_token_liquidity * amount_to_remove / total_supply;

				let updated_total_supply = total_supply - amount_to_remove;

				let mut updated_user_liquidity_balance = LiquidityBalance::get(&sa.sender);
				updated_user_liquidity_balance = updated_user_liquidity_balance - amount_to_remove;

				let user_eth_balance = TokenBalance::get(&sa.sender);
				let user_token_balance = EthBalance::get(&sa.sender);

				LiquidityBalance::insert(&sa.sender, updated_user_liquidity_balance);
				PoolETHLiquidity::set(pool_eth_liquidity - amount_eth_to_return);
				PoolTokenLiquidity::set(pool_token_liquidity - amount_token_to_return);
				TotalSupply::set(updated_total_supply);

				EthBalance::insert(&sa.sender, &user_eth_balance);
				TokenBalance::insert(&sa.sender, &user_token_balance);
				Ok(ContractEvent::RemoveLiquidity(sa.sender.encode(), amount_to_remove.into()))
			}

			/// ==================== ::CONTRACT HELPERS:: ==========================
			fn get_min_user_token_balance(sa: &SenderAmount) -> Uint256 {
				let user_token_balance = TokenBalance::get(&sa.sender);
				cmp::min(sa.amount, user_token_balance)
			}

			fn get_min_user_eth_balance(sa: &SenderAmount) -> Uint256 {
				let user_eth_balance = EthBalance::get(&sa.sender);
				cmp::min(sa.amount, user_eth_balance)
			}

			fn get_min_user_liquidity_balance(sa: &SenderAmount) -> Uint256 {
				let user_liquidity_token_balance = LiquidityBalance::get(&sa.sender);
				cmp::min(sa.amount, user_liquidity_token_balance)
			}

			fn get_ratio() -> Uint256 {
				let token_liquidity = PoolTokenLiquidity::get();
				let eth_liquidity = PoolETHLiquidity::get();
				if eth_liquidity.clone() == Uint256::from(0) {
					return INITIAL_RATIO.into();
				}

				token_liquidity / eth_liquidity
			}

        	// Get block number of incoming message
        	let block_to_sync = be.block_number;
        	let last_synced_block = EthLastSyncedBlock::get();

        	// Compare with last synced block on-chain
        	// Adding new info only if it's greater, otherwise finish with error
        	// It allow to update only the next block,
        	if last_synced_block == 0 || block_to_sync == last_synced_block + 1 {

        		// Iterate by all commands in block
				for cmd in be.methods.clone() {
					let res = match cmd {
						DepositToken(sa) =>  deposit_token(sa),
						DepositETH(sa) => deposit_eth(sa),
						WithdrawETH(sa) => withdraw_eth(sa),
						WithdrawToken(sa) => withdraw_token(sa),
						SwapToToken(sa) => swap_to_token(sa),
						SwapToETH(sa) => swap_to_eth(sa),
						AddLiquidity(sa) => add_liquidity(sa),
						RemoveLiquidity(sa) => remove_liquidity(sa),
					};

					match res {
						Ok(e) => match e {
							ContractEvent::DepositedToken(s, a) => Self::deposit_event(RawEvent::DepositedToken(s, a)),
							ContractEvent::DepositedETH(s, a) => Self::deposit_event(RawEvent::DepositedETH(s, a)),
							ContractEvent::WithdrawToken(s, a) => Self::deposit_event(RawEvent::WithdrawToken(s, a)),
							ContractEvent::WithdrawETH(s, a) => Self::deposit_event(RawEvent::WithdrawETH(s, a)),
							ContractEvent::SwapToToken(s, a) => Self::deposit_event(RawEvent::SwapToToken(s, a)),
							ContractEvent::SwapToETH(s, a) => Self::deposit_event(RawEvent::SwapToETH(s, a)),
							ContractEvent::AddLiquidity(s, a) => Self::deposit_event(RawEvent::AddLiquidity(s, a)),
							ContractEvent::RemoveLiquidity(s, a) => Self::deposit_event(RawEvent::RemoveLiquidity(s, a)),
						}
						Err(err) => {
							debug::error!("{:}", err.0);
						}
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
		fn offchain_worker(_block_number: T::BlockNumber) {
			Self::offchain_eth_sync();
			}
    }
}

