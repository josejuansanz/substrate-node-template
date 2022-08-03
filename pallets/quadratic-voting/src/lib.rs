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

	const NUM_INITIAL_TOKENS: i8 = 100;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Token: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Proposals<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, u8, i8), OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Users<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, i8, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type UsersVotes<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (Vec<u8>, i8), OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a proposal has been added.
		/// Parameters: [who, proposal]
		ProposalAdded(T::AccountId, Vec<u8>),

		/// Event emitted when a user has registered in a proposal.
		/// Parameters: [who]
		UserRegistered(T::AccountId),

		/// Event emitted when a user has unregistered in a proposal.
		/// Parameters: [who]
		UserUnregistered(T::AccountId),
		
		/// Event emitted when a vote has been deposited.
		/// Parameters: [who, proposal, num_votes]
		VoteDeposited(T::AccountId, Vec<u8>, i8),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		EmptyProposal,
		InvalidProposal,
		NotRegistered,
		NotEnoughTokens,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(1_000)]
		pub fn set_proposal(
			origin: OriginFor<T>,
			proposal: Vec<u8>,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;
			
			ensure!(!proposal.is_empty(), Error::<T>::EmptyProposal);

			// Store the proposal with the proposer and active status.
			let active = 1;
			Proposals::<T>::insert(&proposal, (&proposer, active, 0));

			// Emit an event that the user was registered.
			Self::deposit_event(Event::ProposalAdded(proposer, proposal));

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn register(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;

			// Store the user and initialize his tokens
			Users::<T>::insert(&user, NUM_INITIAL_TOKENS);

			// Lock user's tokens
			// Try to reserve funds, and fail fast if the user can't afford it
			T::Token::reserve(&user, 1_000u32.into())?;

			// Emit an event that the user was registered.
			Self::deposit_event(Event::UserRegistered(user));

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn unregister(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;

			// Verify that the user is registered
			ensure!(Users::<T>::contains_key(&user), Error::<T>::NotRegistered);
			
			// Remove the user
			Users::<T>::remove(&user);

			// Remove the user's votes from all the proposals
			for (who, (proposal, num_votes)) in UsersVotes::<T>::iter() {
				if who == user {
					let (proposer, active, proposal_votes) = Proposals::<T>::get(&proposal).expect("Already check that proposal exists.");
					let total_current_votes = proposal_votes - num_votes;
					Proposals::<T>::insert(&proposal, (&proposer, active, total_current_votes));
				}
			}

			// Unlock user's tokens
			// Attempt to unreserve the funds from the user.
			let _ = T::Token::unreserve(&user, 1_000u32.into());

			// Emit an event that the user was unregistered.
			Self::deposit_event(Event::UserUnregistered(user));

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn vote(
			origin: OriginFor<T>,
			num_votes: i8,
			proposal: Vec<u8>,
		) -> DispatchResult {
			let voter = ensure_signed(origin)?;
			
			// Verify that the proposal exists.
			ensure!(Proposals::<T>::contains_key(&proposal), Error::<T>::InvalidProposal);

			// Verify that the user is registered in the voting.
			ensure!(Users::<T>::contains_key(&voter), Error::<T>::NotRegistered);

			let current_num_votes;
			if UsersVotes::<T>::contains_key(&voter) {
				let (_, previous_num_votes) = UsersVotes::<T>::get(&voter).expect("Already check that value exists.");
				current_num_votes = previous_num_votes + num_votes;
			} else {
				current_num_votes = num_votes;
			}

			let current_vote_tokens = current_num_votes.pow(2);

			let current_user_tokens = NUM_INITIAL_TOKENS - current_vote_tokens;

			// Validate the user has enough tokens to deposit his vote
			ensure!(current_user_tokens >= 0, Error::<T>::NotEnoughTokens);
			
			// Update the user's remaining tokens
			Users::<T>::insert(&voter, current_user_tokens);

			// Update user's votes
			if current_num_votes == 0 {
				UsersVotes::<T>::remove(&voter);
			} else {
				UsersVotes::<T>::insert(&voter, (&proposal, current_num_votes));
			}
			
			// Update proposal total votes
			let (proposer, active, previous_proposal_votes) = Proposals::<T>::get(&proposal).expect("Already check that proposal exists.");
			let total_current_votes = previous_proposal_votes + num_votes;
			Proposals::<T>::insert(&proposal, (&proposer, active, total_current_votes));
			
			// Emit an event that the vote was deposited.
			Self::deposit_event(Event::VoteDeposited(voter, proposal, num_votes));

			Ok(())
		}

	}
}
