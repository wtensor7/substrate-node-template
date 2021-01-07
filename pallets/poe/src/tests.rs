use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![1,2,3,4];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		assert_eq!(Proofs::<Test>::get(&claim), (1, frame_system::Module::<Test>::block_number()));
	})
}

#[test]
fn create_claim_failed_when_claim_already_exists() {
	new_test_ext().execute_with(|| {
		let claim = vec![1,2,3,4];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyClaimed
		);
	})
}

#[test]
fn revoke_claim_when_not_proof(){
	new_test_ext().execute_with(|| {
		let claim = vec![1,2,3,4];

		// 撤销一个不存在的凭证
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::NoSuchProof
		);
		
	})
}

#[test]
fn revoke_claim_success() {
	new_test_ext().execute_with(|| {
		let claim = vec![1,2,3,4];

		// 创建凭证
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		// 检查凭证是否撤销成功
		assert_eq!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()), Ok(()));
	})
}

#[test]
fn transfer_claim_success(){
	new_test_ext().execute_with(|| {
		let claim = vec![1,2,3,4];

		// 创建凭证
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		// 检查是否转移成功
		assert_eq!(
			PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2u64),
			Ok(())
		);

	})
}

#[test]
fn transfer_claim_not_proof(){
	new_test_ext().execute_with(|| {
		let claim = vec![1,2,3,4];
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2u64),
			Error::<Test>::ClaimNotExist
		);
	})
}
#[test]
fn transfer_claim_not_owner(){
	new_test_ext().execute_with(|| {
		let claim = vec![1,2,3,4];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(2), claim.clone(), 2u64),
			Error::<Test>::NotProofOwner
		);

	})
}

// 创建存证限制大小
#[test]
fn create_claim_proof_too_long(){
	new_test_ext().execute_with(|| {
		// 限制len大小
		let claim = vec![1, 2, 3, 4, 5, 6, 7];
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()), 
			Error::<Test>::ProofTooLong
		);
	})
}
