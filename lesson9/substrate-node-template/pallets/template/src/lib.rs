#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{debug, decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
    offchain as rt_offchain,
    offchain::storage::StorageValueRef
};
use alt_serde::{Deserialize, Deserializer};
use sp_std::prelude::*;
use sp_std::str;
use sp_io;
use codec::{Decode, Encode};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
	// It is important to update your storage name so that your pallet's
	// storage items are isolated from other pallets.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Numbers get(fn number): map hasher(blake2_128_concat) u64 => u64;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Just a dummy event.
		/// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		/// To emit this event, we call the deposit function, from our runtime functions
		SomethingStored(u32, AccountId),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,
		HttpFetchingError,
		InvalidResponse,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

		#[weight = 10_000]
		pub fn save_number(origin, number: u32) -> dispatch::DispatchResult {
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let who = ensure_signed(origin)?;

			/*******
			 * 学员们在这里追加逻辑
			 *******/
			// Number::insert(index, number);
			Ok(())
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug::info!("Entering off-chain workers");

			/*******
			 * 学员们在这里追加逻辑
			 *******/
			 Self::fetch_eth_price();
		}

	}
}

// Refer to https://min-api.cryptocompare.com/documentation
pub const CC_ETH_USD: &[u8] = b"https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=USD";

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct EthPrice {
	// #[serde(deserialize_with = "de_string_to_bytes")]
	price: u32
}

// pub fn de_string_to_bytes<de, D>(de: D) -> Result<Vec<u8>>, D::Error>
// where D::Deserializer<de>,
// {
// 	let s: &str = Deserialize::deserialize(de)?;
// 	OK(s.as_bytes().to_vec())
// }

impl <T: Trait> Module<T> {
	fn fetch_eth_price() {
		let s_info = StorageValueRef::persistent(b"offchain::eth-price");

		if let Some(Some(eth_price)) = s_info.get::<EthPrice>() {
			debug::info!("cached eth-usd price: {:?}", eth_price.price);
		}

		match Self::fetch_eth_price_cc() {
			Ok(info) => {
				s_info.set(&info);
				debug::info!("fetch_eth_price: {:?}", info.price)
			}
			Err(_) => {
				debug::error!("Failed to fetch price")
			}
		}

	}

	fn fetch_eth_price_cc() -> Result<EthPrice, Error<T>> {
		let url_bytes = CC_ETH_USD.to_vec();
		let price_url = str::from_utf8(&url_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;

		debug::info!("url {:?}", price_url);

		let request = rt_offchain::http::Request::get(price_url);
		let timeout = sp_io::offchain::timestamp().add(rt_offchain::Duration::from_millis(3000));

		let pending = request
			.deadline(timeout)
			.send()
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

			let response = pending.try_wait(timeout)
			.map_err(|_| <Error<T>>::HttpFetchingError)?
			.map_err(|_| <Error<T>>::HttpFetchingError)?;
		
		if response.code != 200 {
			debug::error!("Unexpected http request status code: {}", response.code);
			return Err(<Error<T>>::HttpFetchingError);
		}

		let response_array = response.body().collect::<Vec<u8>>();

		let resp_str = str::from_utf8(&response_array).map_err(|_| <Error<T>>::HttpFetchingError)?;
		debug::info!("response is {:?}", resp_str);
		
		let eth_price: EthPrice = serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::InvalidResponse)?;

		Ok(eth_price)

	}
}
