use crate::{mock::*};
use frame_support::{assert_ok, assert_noop};
use codec::{Encode};
use sp_runtime::traits::BadOrigin;
use crate as pallet_identity;

#[test]
fn setting_new_identity_works() {
	let account = 0x432445325;
	let name = String::from("Jhonny").encode();
	new_test_ext().execute_with(|| {
		assert_ok!(IdentityVoting::set_identity(Origin::signed(2), account, name.clone()));
		assert_noop!(IdentityVoting::set_identity(Origin::signed(10), account, name.clone()), BadOrigin);
		assert_noop!(IdentityVoting::set_identity(Origin::signed(2), account, Vec::new()), pallet_identity::Error::<Test>::EmptyName);
	});
}

#[test]
fn removing_identity_works() {
	let account = 0x432445325;
	new_test_ext().execute_with(|| {
		assert_noop!(IdentityVoting::remove_identity(Origin::signed(10), account, ), BadOrigin);
		assert_noop!(IdentityVoting::remove_identity(Origin::signed(2), account), pallet_identity::Error::<Test>::AccountHasNoIdentity);
	});
}