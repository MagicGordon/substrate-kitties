use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			KittiesModule::kitties_count(),
			None
		);
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_eq!(
			KittiesModule::kitties_count(),
			Some(2)
		);
		assert_eq!(
			<Test as super::Config>::Currency::reserved_balance(1),
			100000000000000
		);
		
		assert_eq!(
			last_event(),
			<Test as super::Config>::Event::KittiesModule(crate::Event::KittiesCreate(1, 1))
		);
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_eq!(
			KittiesModule::kitties_count(),
			Some(3)
		);
		assert_eq!(
			<Test as super::Config>::Currency::reserved_balance(1),
			200000000000000
		);

	});
}

#[test]
fn create_reserve_failed() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			KittiesModule::create(Origin::signed(3)),
			Error::<Test>::ReserveFailed
		);
	});
}

#[test]
fn transfer_workers(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::transfer(Origin::signed(1), 2, 1));
		assert_eq!(
			last_event(),
			<Test as super::Config>::Event::KittiesModule(crate::Event::KittyTransfer(1, 2, 1))
		);
	});
}


#[test]
fn transfer_not_owner(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1), 2, 2),
			Error::<Test>::NotOwner
		);
	});
	
}



#[test]
fn breed_workers(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::breed(Origin::signed(1), 1, 2));
		assert_eq!(
			KittiesModule::kitties_count(),
			Some(4)
		);
		assert_eq!(
			last_event(),
			<Test as super::Config>::Event::KittiesModule(crate::Event::KittiesCreate(1, 3))
		);
	});
}

#[test]
fn breed_same_parent_index(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::breed(Origin::signed(1), 1, 1),
			Error::<Test>::SameParentIndex
		);
	});
}

#[test]
fn breed_invalid_kitty_index(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::breed(Origin::signed(1), 1, 2),
			Error::<Test>::InvalidKittyIndex
		);
	});
}

#[test]
fn buy_workers(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::buy(Origin::signed(2), 1, 100 * 1_000 * 1_000_000_000));
		assert_eq!(
			<Test as super::Config>::Currency::free_balance(1),
			10100 * 1_000 * 1_000_000_000
		);
		assert_eq!(
			<Test as super::Config>::Currency::free_balance(2),
			9900 * 1_000 * 1_000_000_000
		);
		assert_eq!(
			last_event(),
			<Test as super::Config>::Event::KittiesModule(crate::Event::BuyKitty(1, 2, 100 * 1_000 * 1_000_000_000, 1))
		);
	});
	
}

#[test]
fn buy_invalid_kitty_index(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::buy(Origin::signed(2), 2, 100 * 1_000 * 1_000_000_000),
			Error::<Test>::InvalidKittyIndex
		);
	});
}

#[test]
fn buy_buyer_is_owner(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::buy(Origin::signed(1), 1, 100 * 1_000 * 1_000_000_000),
			Error::<Test>::BuyerIsOwner
		);
	});
}

#[test]
fn sell_workers(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::sell(Origin::signed(1), 1, 2, 100 * 1_000 * 1_000_000_000));
		assert_eq!(
			<Test as super::Config>::Currency::free_balance(1),
			10100 * 1_000 * 1_000_000_000
		);
		assert_eq!(
			<Test as super::Config>::Currency::free_balance(2),
			9900 * 1_000 * 1_000_000_000
		);
		assert_eq!(
			last_event(),
			<Test as super::Config>::Event::KittiesModule(crate::Event::SellKitty(1, 2, 100 * 1_000 * 1_000_000_000, 1))
		);
	});
	
}

#[test]
fn sell_invalid_kitty_index(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::sell(Origin::signed(2), 2, 2, 100 * 1_000 * 1_000_000_000),
			Error::<Test>::InvalidKittyIndex
		);
	});
}

#[test]
fn sell_buyer_is_owner(){
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::sell(Origin::signed(2), 1, 1, 100 * 1_000 * 1_000_000_000),
			Error::<Test>::NotOwner
		);
	});
}
