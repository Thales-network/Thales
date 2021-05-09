use std::{convert::TryFrom, str::FromStr};

use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use hex_literal::hex;
use pallet_evm::{
    Account as EVMAccount, EnsureAddressTruncated, HashedAddressMapping, AddressMapping,
};
use sp_core::{H160, crypto::Ss58AddressFormat};
use sp_core::crypto::Ss58Codec;
use sp_core::{ed25519, hexdisplay::HexDisplay, Pair, Public};
use sp_runtime::{AccountId32, traits::{BlakeTwo256, IdentifyAccount}};

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
        // Read pallet storage and assert an expected result.
        assert_eq!(TemplateModule::something(), Some(42));
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            TemplateModule::cause_error(Origin::signed(1)),
            Error::<Test>::NoneValue
        );
    });
}

#[test]
fn to_ss58check_with_version() {
    let (pair, _, _) = ed25519::Pair::generate_with_phrase(None);
    let max_u16 = u16::max_value();
    println!("max_u16 = {}", max_u16);
    for number in (0..65535).rev() {
        let address = pair
            .public()
            .into_account()
            .to_ss58check_with_version(Ss58AddressFormat::Custom(number));
        if address.starts_with("n") {
            println!("number: {}", number);
            println!("address: {}", address);
        }
    }
}

#[test]
fn to_ss58check_with_153() {
    // 153, 154, 155, 157, 158, 159, 160, 161
    for number in 0..500 {
        let (pair, _, _) = ed25519::Pair::generate_with_phrase(None);
        let address = pair
            .public()
            .into_account()
            .to_ss58check_with_version(Ss58AddressFormat::Custom(153));
        if address.starts_with("n") {
            println!("address: {}", address);
        } else {
            panic!("addre failed");
        }
    }
}

#[test]
fn total_test() {
    let initial_balance = 10u128.pow(8 + 9);
    println!("initial_balance = {}", initial_balance);
    // 1000000000_00000000
}

// #[test]
// fn pk_to_ss58() {
//     let seed = hex![""];
//     let pair = ed25519::Pair::from_seed_slice(seed.as_ref()).unwrap();
//     let pubkey = pair.public();
//     let addr = pubkey.into_account().to_ss58check();
//     println!("addr: {}", addr);
// }

#[test]
fn eth_address_to_sub_address() {
    type AddressMapping = HashedAddressMapping<BlakeTwo256>;
    // 5GJGouYhoriALGunyg65qBbqg5SnuRpJdpna2Jwf6uJBvc62
    let eth_addr = H160::from_str("1DfeF571E6E4418a3E170889DE0e4E784FeA1E4a").unwrap();
    let sub_addr: AccountId32 = AddressMapping::into_account_id(eth_addr);
    println!("addr: {}", sub_addr.to_ss58check());
}