#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Token: ReservableCurrency<Self::AccountId>;

		type ForceOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Identities<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u8>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when an identity has been added.
		/// Parameters: [who, name]
		IdentityAdded(T::AccountId, Vec<u8>),

		/// Event emitted when an identity has been removed.
		/// Parameters: [who]
		IdentityRemoved(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		EmptyName,
		AccountHasNoIdentity,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(1_000)]
		pub fn set_identity(
			origin: OriginFor<T>,
			account: T::AccountId,
			name: Vec<u8>,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			
			ensure!(!name.is_empty(), Error::<T>::EmptyName);

			// Store the identity with the name.
			Identities::<T>::insert(&account, &name);

			// Lock user's tokens
			// Try to reserve funds, and fail fast if the user can't afford it
			T::Token::reserve(&account, 1_000u32.into())?;

			// Emit an event that the identity was added.
			Self::deposit_event(Event::IdentityAdded(account, name));

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn remove_identity(
			origin: OriginFor<T>,
			account: T::AccountId,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			
			// Verify that the user is registered.
			ensure!(Identities::<T>::contains_key(&account), Error::<T>::AccountHasNoIdentity);

			// Remove the identity.
			Identities::<T>::remove(&account);

			// Unlock user's tokens
			// Attempt to unreserve the funds from the user.
			let _ = T::Token::unreserve(&account, 1_000u32.into());

			// Emit an event that the user was registered.
			Self::deposit_event(Event::IdentityRemoved(account));

			Ok(())
		}

	}
}
