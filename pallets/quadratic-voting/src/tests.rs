use crate::{mock::*};
use frame_support::{assert_ok, assert_noop};
use codec::{Encode};
use crate as pallet_quadratic_voting;

#[test]
fn setting_new_proposal_works() {
	let proposal = String::from("This is a test proposal").encode();
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVoting::set_proposal(Origin::signed(1), proposal.clone()));
		assert_noop!(QuadraticVoting::set_proposal(Origin::signed(1), Vec::new()), pallet_quadratic_voting::Error::<Test>::EmptyProposal);
	});
}

#[test]
fn registering_user_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(QuadraticVoting::register(Origin::signed(1)));
	});
}

#[test]
fn unregistering_user_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(QuadraticVoting::unregister(Origin::signed(1)), pallet_quadratic_voting::Error::<Test>::NotRegistered);
	});
}

#[test]
fn voting_works() {
	let num_votes = 1;
	let proposal = String::from("This is a test proposal").encode();
	new_test_ext().execute_with(|| {
		assert_noop!(QuadraticVoting::vote(Origin::signed(1), num_votes, proposal), pallet_quadratic_voting::Error::<Test>::InvalidProposal);
	});
}
