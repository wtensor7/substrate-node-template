use crate::{Event, Error, mock::*};
use frame_support::{assert_noop, assert_ok, traits::{OnFinalize, OnInitialize}};

// 随机 需要之前有块
fn run_to_block( n: u64) {
	while System::block_number() < n {
		KittiesModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number()+1);
		System::on_initialize(System::block_number());
		KittiesModule::on_initialize(System::block_number());
	}
}

// 测试创建一个 Kitty
#[test]
fn create_kitty_works(){
	new_test_ext().execute_with(|| {
		run_to_block(5);
		assert_ok!(KittiesModule::create( Origin::signed(1)) );

		//触发两个事件，这里只监控第二个
		assert_eq!(
			System::events()[1].event,
			TestEvent::kitties( Event::<Test>::Created( 1u64 , 0) )
		);
	})
}

// 测试创建 Kitty 余额不足
#[test]
fn create_kitty_failed_when_not_enough_money(){
	new_test_ext().execute_with(|| {
		run_to_block(5);
		assert_noop!(KittiesModule::create( Origin::signed(9)) , Error::<Test>::MoneyNotEnough);
	})
}

// 测试转让 Kitty 成功
#[test]
fn transfer_kitty_success(){
	new_test_ext().execute_with(|| {
		run_to_block(5);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_ok!(KittiesModule::transfer( Origin::signed(1), 2, 0 ) );

		// 总共会触发个五个事件
		assert_eq!(
			System::events()[4].event,
			TestEvent::kitties( Event::<Test>::Transferred( 1u64 ,2u64, 0) )
		);
	});
}

// 测试转让 Kitty 不存在
#[test]
fn transfer_kitty_failed_when_not_exists(){
	new_test_ext().execute_with(|| {
		assert_noop!(KittiesModule::transfer( Origin::signed(1), 2, 0 ) , Error::<Test>::KittyNotExists);
	})
}

// 测试转让非kitty所有者
#[test]
fn transfer_kitty_failed_when_not_owner(){
	new_test_ext().execute_with(|| {
		run_to_block(5);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_noop!(
			KittiesModule::transfer( Origin::signed(2), 3, 0 ) 
		, Error::<Test>::KittyNotOwner);
	})
}

// 测试自己转让给自己
#[test]
fn transfer_kitty_when_transfer_self(){
	new_test_ext().execute_with(|| {
		run_to_block(5);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_noop!(
			KittiesModule::transfer( Origin::signed(1), 1, 0 ) , 
			Error::<Test>::TransferSelf);
	})
}
// 测试繁殖成功
#[test]
fn breed_kitty_success(){
	new_test_ext().execute_with(|| {
		run_to_block(5);
		let _ = KittiesModule::create( Origin::signed(1) );
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_ok!( KittiesModule::breed( Origin::signed(1), 0, 1 ) );

		assert_eq!(
			System::events()[1].event,
			TestEvent::kitties( Event::<Test>::Created( 1u64 , 0) )
		);
	});
}


// 测试kitty不存在
#[test]
fn breed_kitty_when_not_exists(){
	new_test_ext().execute_with(|| {
		assert_noop!( KittiesModule::breed( Origin::signed(1), 0, 1 ) , Error::<Test>::KittyNotExists);
	})
}

// 非kitty所有者
#[test]
fn breed_kitty_when_not_owner(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_noop!( KittiesModule::breed( Origin::signed(2), 0, 1) , Error::<Test>::KittyNotOwner);
	})
}


// 测试同一只kitty繁殖
#[test]
fn breed_kitty_when_parents_same(){
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = KittiesModule::create( Origin::signed(1) );

		assert_noop!( KittiesModule::breed( Origin::signed(1), 0, 0 ) , Error::<Test>::RequiredDiffrentParent);
	})
}