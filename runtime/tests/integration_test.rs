// Copyright 2019-2021 Thales Inc.
// This file is part of Thales.

// Thales is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Thales is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Thales.  If not, see <http://www.gnu.org/licenses/>.

//! Thales Runtime Integration Tests

#![cfg(test)]

use cumulus_primitives_parachain_inherent::ParachainInherentData;
use evm::{Context, ExitSucceed};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::Dispatchable,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
};
use thales_runtime::{
	AccountId, AuthorInherent, Balance, Balances, Call, Event, InflationInfo, ParachainStaking,
	Range, Runtime, System, GLMR,
};
use pallet_evm::PrecompileSet;
use parachain_staking::Bond;
use precompiles::ThalesPrecompiles;
use sp_core::{H160, U256};
use sp_runtime::{DispatchError, Perbill};

fn run_to_block(n: u32) {
	while System::block_number() < n {
		AuthorInherent::on_finalize(System::block_number());
		ParachainStaking::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		AuthorInherent::on_initialize(System::block_number());
	}
}

fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	// [collator, amount]
	collators: Vec<(AccountId, Balance)>,
	// [nominator, collator, nomination_amount]
	nominators: Vec<(AccountId, AccountId, Balance)>,
	// per-round inflation config
	inflation: InflationInfo<Balance>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			nominators: vec![],
			collators: vec![],
			inflation: InflationInfo {
				expect: Range {
					min: 100_000 * GLMR,
					ideal: 200_000 * GLMR,
					max: 500_000 * GLMR,
				},
				// not used
				annual: Range {
					min: Perbill::from_percent(50),
					ideal: Perbill::from_percent(50),
					max: Perbill::from_percent(50),
				},
				// unrealistically high parameterization, only for testing
				round: Range {
					min: Perbill::from_percent(5),
					ideal: Perbill::from_percent(5),
					max: Perbill::from_percent(5),
				},
			},
		}
	}
}

impl ExtBuilder {
	fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	fn with_collators(mut self, collators: Vec<(AccountId, Balance)>) -> Self {
		self.collators = collators;
		self
	}

	fn with_nominators(mut self, nominators: Vec<(AccountId, AccountId, Balance)>) -> Self {
		self.nominators = nominators;
		self
	}

	#[allow(dead_code)]
	fn with_inflation(mut self, inflation: InflationInfo<Balance>) -> Self {
		self.inflation = inflation;
		self
	}

	fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut stakers: Vec<(AccountId, Option<AccountId>, Balance)> = Vec::new();
		for collator in self.collators {
			stakers.push((collator.0, None, collator.1));
		}
		for nominator in self.nominators {
			stakers.push((nominator.0, Some(nominator.1), nominator.2));
		}
		parachain_staking::GenesisConfig::<Runtime> {
			stakers,
			inflation_config: self.inflation,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

const ALICE: [u8; 20] = [4u8; 20];
const BOB: [u8; 20] = [5u8; 20];
const CHARLIE: [u8; 20] = [6u8; 20];
const DAVE: [u8; 20] = [7u8; 20];

fn origin_of(account_id: AccountId) -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::signed(account_id)
}

fn inherent_origin() -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::none()
}

/// Mock the inherent that sets author in `author-inherent`
fn set_author(a: AccountId) {
	assert_ok!(
		Call::AuthorInherent(author_inherent::Call::<Runtime>::set_author(a))
			.dispatch(inherent_origin())
	);
}

/// Mock the inherent that sets validation data in ParachainSystem, which
/// contains the `relay_chain_block_number`, which is used in `author-filter` as a
/// source of randomness to filter valid authors at each block.
fn set_parachain_inherent_data() {
	use cumulus_primitives_core::PersistedValidationData;
	use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
	let (relay_parent_storage_root, relay_chain_state) =
		RelayStateSproofBuilder::default().into_state_root_and_proof();
	let vfp = PersistedValidationData {
		relay_parent_number: 1u32,
		relay_parent_storage_root,
		..Default::default()
	};
	let parachain_inherent_data = ParachainInherentData {
		validation_data: vfp,
		relay_chain_state: relay_chain_state,
		downward_messages: Default::default(),
		horizontal_messages: Default::default(),
	};
	assert_ok!(Call::ParachainSystem(
		cumulus_pallet_parachain_system::Call::<Runtime>::set_validation_data(
			parachain_inherent_data
		)
	)
	.dispatch(inherent_origin()));
}

#[test]
fn join_collator_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 2_000 * GLMR),
			(AccountId::from(CHARLIE), 1_100 * GLMR),
			(AccountId::from(DAVE), 1_000 * GLMR),
		])
		.with_collators(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_nominators(vec![
			(AccountId::from(CHARLIE), AccountId::from(ALICE), 50 * GLMR),
			(AccountId::from(CHARLIE), AccountId::from(BOB), 50 * GLMR),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::join_candidates(origin_of(AccountId::from(ALICE)), 1_000 * GLMR,),
				parachain_staking::Error::<Runtime>::CandidateExists
			);
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					1_000 * GLMR
				),
				parachain_staking::Error::<Runtime>::NominatorExists
			);
			assert!(System::events().is_empty());
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(DAVE)),
				1_000 * GLMR,
			));
			assert_eq!(
				last_event(),
				Event::parachain_staking(parachain_staking::Event::JoinedCollatorCandidates(
					AccountId::from(DAVE),
					1_000 * GLMR,
					3_100 * GLMR
				))
			);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(
				candidates.0[0],
				Bond {
					owner: AccountId::from(ALICE),
					amount: 1_050 * GLMR
				}
			);
			assert_eq!(
				candidates.0[1],
				Bond {
					owner: AccountId::from(BOB),
					amount: 1_050 * GLMR
				}
			);
			assert_eq!(
				candidates.0[2],
				Bond {
					owner: AccountId::from(DAVE),
					amount: 1_000 * GLMR
				}
			);
		});
}

#[test]
fn transfer_through_evm_to_stake() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 3_000 * GLMR)])
		.build()
		.execute_with(|| {
			// Charlie has no balance => fails to stake
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					1_000 * GLMR,
				),
				DispatchError::Module {
					index: 3,
					error: 3,
					message: Some("InsufficientBalance")
				}
			);
			// Alice stakes to become a collator candidate
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(ALICE)),
				1_000 * GLMR,
			));
			// Alice transfer from free balance 1000 GLMR to Bob
			assert_ok!(Balances::transfer(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				2_000 * GLMR,
			));
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 2_000 * GLMR,);
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();
			// Bob transfers 1000 GLMR to Charlie via EVM
			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(BOB),
				AccountId::from(CHARLIE),
				Vec::new(),
				(1_000 * GLMR).into(),
				gas_limit,
				gas_price,
				None
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				1_000 * GLMR,
			);
			// Charlie can stake now
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(CHARLIE)),
				1_000 * GLMR,
			),);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(
				candidates.0[0],
				Bond {
					owner: AccountId::from(ALICE),
					amount: 2_000 * GLMR
				}
			);
			assert_eq!(
				candidates.0[1],
				Bond {
					owner: AccountId::from(CHARLIE),
					amount: 1_000 * GLMR
				}
			);
		});
}

#[test]
fn reward_block_authors() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_nominators(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * GLMR,
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			for x in 2..1201 {
				set_author(AccountId::from(ALICE));
				run_to_block(x);
			}
			// no rewards doled out yet
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1_000 * GLMR,);
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 500 * GLMR,);
			set_author(AccountId::from(ALICE));
			run_to_block(1201);
			// rewards minted and distributed
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1109999999920000000000,
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				539999999960000000000,
			);
		});
}

#[test]
fn join_candidates_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 3_000 * GLMR)])
		.build()
		.execute_with(|| {
			let staking_precompile_address = H160::from_low_u64_be(2048);

			// Alice uses the staking precompile to join as a candidate through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();
			let amount: U256 = (1000 * GLMR).into();

			// Construct the call data (selector, amount)
			let mut call_data = Vec::<u8>::from([0u8; 36]);
			call_data[0..4].copy_from_slice(&hex_literal::hex!("ad76ed5a"));
			amount.to_big_endian(&mut call_data[4..36]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Assert that Alice is now a candidate
			assert!(ParachainStaking::is_candidate(&AccountId::from(ALICE)));

			// Check for the right events.
			let expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Reserved(
					AccountId::from(ALICE),
					1000 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::JoinedCollatorCandidates(
					AccountId::from(ALICE),
					1000 * GLMR,
					1000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn leave_candidates_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * GLMR)])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.build()
		.execute_with(|| {
			let staking_precompile_address = H160::from_low_u64_be(2048);

			// Alice uses the staking precompile to leave_candidates
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct the leave_candidates call data
			let mut call_data = Vec::<u8>::from([0u8; 4]);
			call_data[0..4].copy_from_slice(&hex_literal::hex!("b7694219"));

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let expected_events = vec![
				Event::parachain_staking(parachain_staking::Event::CollatorScheduledExit(
					1,
					AccountId::from(ALICE),
					3,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn go_online_offline_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * GLMR)])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.build()
		.execute_with(|| {
			// Alice is initialized as a candidate
			assert!(ParachainStaking::is_candidate(&AccountId::from(ALICE)));
			let staking_precompile_address = H160::from_low_u64_be(2048);

			// Alice uses the staking precompile to go offline
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct the go_offline call data
			let mut go_offline_call_data = Vec::<u8>::from([0u8; 4]);
			go_offline_call_data[0..4].copy_from_slice(&hex_literal::hex!("767e0450"));

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				go_offline_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut expected_events = vec![
				Event::parachain_staking(parachain_staking::Event::CollatorWentOffline(
					1,
					AccountId::from(ALICE),
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);

			// Construct the go_online call data
			let mut go_online_call_data = Vec::<u8>::from([0u8; 4]);
			go_online_call_data[0..4].copy_from_slice(&hex_literal::hex!("d2f73ceb"));

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				go_online_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut new_events = vec![
				Event::parachain_staking(parachain_staking::Event::CollatorBackOnline(
					1,
					AccountId::from(ALICE),
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];
			expected_events.append(&mut new_events);

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn candidate_bond_more_less_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 3_000 * GLMR)])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.build()
		.execute_with(|| {
			// Alice is initialized as a candidate
			assert!(ParachainStaking::is_candidate(&AccountId::from(ALICE)));
			let staking_precompile_address = H160::from_low_u64_be(2048);

			// Alice uses the staking precompile to bond more
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct the candidate_bond_more call
			let mut bond_more_call_data = Vec::<u8>::from([0u8; 36]);
			bond_more_call_data[0..4].copy_from_slice(&hex_literal::hex!("c57bd3a8"));
			let bond_more_amount: U256 = (1000 * GLMR).into();
			bond_more_amount.to_big_endian(&mut bond_more_call_data[4..36]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				bond_more_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Reserved(
					AccountId::from(ALICE),
					1_000 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::CollatorBondedMore(
					AccountId::from(ALICE),
					1_000 * GLMR,
					2_000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);

			// Construct the go_online call data
			let mut bond_less_call_data = Vec::<u8>::from([0u8; 36]);
			bond_less_call_data[0..4].copy_from_slice(&hex_literal::hex!("289b6ba7"));
			let bond_less_amount: U256 = (500 * GLMR).into();
			bond_less_amount.to_big_endian(&mut bond_less_call_data[4..36]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				bond_less_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut new_events = vec![
				Event::pallet_balances(pallet_balances::Event::Unreserved(
					AccountId::from(ALICE),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::CollatorBondedLess(
					AccountId::from(ALICE),
					2_000 * GLMR,
					1_500 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];
			expected_events.append(&mut new_events);

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn nominate_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 3_000 * GLMR),
			(AccountId::from(BOB), 3_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.build()
		.execute_with(|| {
			let staking_precompile_address = H160::from_low_u64_be(2048);

			// Bob uses the staking precompile to nominate Alice through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();
			let nomination_amount: U256 = (1000 * GLMR).into();

			// Construct the call data (selector, collator, nomination amount)
			let mut call_data = Vec::<u8>::from([0u8; 68]);
			call_data[0..4].copy_from_slice(&hex_literal::hex!("82f2c8df"));
			call_data[16..36].copy_from_slice(&ALICE);
			nomination_amount.to_big_endian(&mut call_data[36..68]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(BOB),
				staking_precompile_address,
				call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Assert that Bob is now nominating Alice
			assert!(ParachainStaking::is_nominator(&AccountId::from(BOB)));

			// Check for the right events.
			let expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Reserved(
					AccountId::from(BOB),
					1000 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::Nomination(
					AccountId::from(BOB),
					1000 * GLMR,
					AccountId::from(ALICE),
					2000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn leave_nominators_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
			(AccountId::from(CHARLIE), 1_500 * GLMR),
		])
		.with_collators(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_nominators(vec![
			(AccountId::from(CHARLIE), AccountId::from(ALICE), 500 * GLMR),
			(AccountId::from(CHARLIE), AccountId::from(BOB), 500 * GLMR),
		])
		.build()
		.execute_with(|| {
			// Charlie is initialized as a nominator
			assert!(ParachainStaking::is_nominator(&AccountId::from(CHARLIE)));
			let staking_precompile_address = H160::from_low_u64_be(2048);

			// Charlie uses staking precompile to leave nominator set
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct leave_nominators call
			let mut call_data = Vec::<u8>::from([0u8; 4]);
			call_data[0..4].copy_from_slice(&hex_literal::hex!("e8d68a37"));

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(CHARLIE),
				staking_precompile_address,
				call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Charlie is no longer a nominator
			assert!(!ParachainStaking::is_nominator(&AccountId::from(CHARLIE)));

			// Check for the right events.
			let expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Unreserved(
					AccountId::from(CHARLIE),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominatorLeftCollator(
					AccountId::from(CHARLIE),
					AccountId::from(ALICE),
					500 * GLMR,
					1_000 * GLMR,
				)),
				Event::pallet_balances(pallet_balances::Event::Unreserved(
					AccountId::from(CHARLIE),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominatorLeftCollator(
					AccountId::from(CHARLIE),
					AccountId::from(BOB),
					500 * GLMR,
					1_000 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominatorLeft(
					AccountId::from(CHARLIE),
					1_000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn revoke_nomination_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
			(AccountId::from(CHARLIE), 1_500 * GLMR),
		])
		.with_collators(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_nominators(vec![
			(AccountId::from(CHARLIE), AccountId::from(ALICE), 500 * GLMR),
			(AccountId::from(CHARLIE), AccountId::from(BOB), 500 * GLMR),
		])
		.build()
		.execute_with(|| {
			// Charlie is initialized as a nominator
			assert!(ParachainStaking::is_nominator(&AccountId::from(CHARLIE)));
			let staking_precompile_address = H160::from_low_u64_be(2048);

			// Charlie uses staking precompile to revoke nomination
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct revoke_nomination call
			let mut call_data = Vec::<u8>::from([0u8; 36]);
			call_data[0..4].copy_from_slice(&hex_literal::hex!("4b65c34b"));
			call_data[16..36].copy_from_slice(&ALICE);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(CHARLIE),
				staking_precompile_address,
				call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Charlie is still a nominator because only nomination to Alice was revoked
			assert!(ParachainStaking::is_nominator(&AccountId::from(CHARLIE)));

			// Check for the right events.
			let expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Unreserved(
					AccountId::from(CHARLIE),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominatorLeftCollator(
					AccountId::from(CHARLIE),
					AccountId::from(ALICE),
					500 * GLMR,
					1_000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn nominator_bond_more_less_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_500 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_nominators(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * GLMR,
		)])
		.build()
		.execute_with(|| {
			// Bob is initialized as a nominator
			assert!(ParachainStaking::is_nominator(&AccountId::from(BOB)));
			let staking_precompile_address = H160::from_low_u64_be(2048);

			// Alice uses the staking precompile to bond more
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct the nominator_bond_more call
			let mut bond_more_call_data = Vec::<u8>::from([0u8; 68]);
			bond_more_call_data[0..4].copy_from_slice(&hex_literal::hex!("971d44c8"));
			bond_more_call_data[16..36].copy_from_slice(&ALICE);
			let bond_more_amount: U256 = (500 * GLMR).into();
			bond_more_amount.to_big_endian(&mut bond_more_call_data[36..68]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(BOB),
				staking_precompile_address,
				bond_more_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Reserved(
					AccountId::from(BOB),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominationIncreased(
					AccountId::from(BOB),
					AccountId::from(ALICE),
					1_500 * GLMR,
					2_000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);

			// Construct the go_online call data
			let mut bond_less_call_data = Vec::<u8>::from([0u8; 68]);
			bond_less_call_data[0..4].copy_from_slice(&hex_literal::hex!("f6a52569"));
			bond_less_call_data[16..36].copy_from_slice(&ALICE);
			let bond_less_amount: U256 = (500 * GLMR).into();
			bond_less_amount.to_big_endian(&mut bond_less_call_data[36..68]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(BOB),
				staking_precompile_address,
				bond_less_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut new_events = vec![
				Event::pallet_balances(pallet_balances::Event::Unreserved(
					AccountId::from(BOB),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominationDecreased(
					AccountId::from(BOB),
					AccountId::from(ALICE),
					2_000 * GLMR,
					1_500 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];
			expected_events.append(&mut new_events);

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn is_nominator_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_500 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_nominators(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * GLMR,
		)])
		.build()
		.execute_with(|| {
			// Confirm Bob is initialized as a nominator directly
			assert!(ParachainStaking::is_nominator(&AccountId::from(BOB)));

			let staking_precompile_address = H160::from_low_u64_be(2048);

			// Construct the input data to check if Bob is a nominator
			let mut bob_input_data = Vec::<u8>::from([0u8; 36]);
			bob_input_data[0..4].copy_from_slice(&hex_literal::hex!("8e5080e7"));
			bob_input_data[16..36].copy_from_slice(&BOB);

			// Expected result is an EVM boolean true which is 256 bits long.
			let mut expected_bytes = Vec::from([0u8; 32]);
			expected_bytes[31] = 1;
			let expected_true_result = Some(Ok((ExitSucceed::Returned, expected_bytes, 0)));

			// Assert precompile reports Bob is a nominator
			assert_eq!(
				ThalesPrecompiles::<Runtime>::execute(
					staking_precompile_address,
					&bob_input_data,
					None, // target_gas is not necessary right now because consumed none now
					&Context {
						// This context copied from Sacrifice tests, it's not great.
						address: Default::default(),
						caller: Default::default(),
						apparent_value: From::from(0),
					}
				),
				expected_true_result
			);

			// Construct the input data to check if Charlie is a nominator
			let mut charlie_input_data = Vec::<u8>::from([0u8; 36]);
			charlie_input_data[0..4].copy_from_slice(&hex_literal::hex!("8e5080e7"));
			charlie_input_data[16..36].copy_from_slice(&CHARLIE);

			// Expected result is an EVM boolean false which is 256 bits long.
			expected_bytes = Vec::from([0u8; 32]);
			let expected_false_result = Some(Ok((ExitSucceed::Returned, expected_bytes, 0)));

			// Assert precompile also reports Charlie as not a nominator
			assert_eq!(
				ThalesPrecompiles::<Runtime>::execute(
					staking_precompile_address,
					&charlie_input_data,
					None,
					&Context {
						// This context copied from Sacrifice tests, it's not great.
						address: Default::default(),
						caller: Default::default(),
						apparent_value: From::from(0),
					}
				),
				expected_false_result
			);
		})
}

#[test]
fn is_candidate_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.build()
		.execute_with(|| {
			// Confirm Alice is initialized as a candidate directly
			assert!(ParachainStaking::is_candidate(&AccountId::from(ALICE)));

			let staking_precompile_address = H160::from_low_u64_be(2048);

			// Construct the input data to check if Alice is a candidate
			let mut alice_input_data = Vec::<u8>::from([0u8; 36]);
			alice_input_data[0..4].copy_from_slice(&hex_literal::hex!("8545c833"));
			alice_input_data[16..36].copy_from_slice(&ALICE);

			// Expected result is an EVM boolean true which is 256 bits long.
			let mut expected_bytes = Vec::from([0u8; 32]);
			expected_bytes[31] = 1;
			let expected_true_result = Some(Ok((ExitSucceed::Returned, expected_bytes, 0)));

			// Assert precompile reports Alice is a collator candidate
			assert_eq!(
				ThalesPrecompiles::<Runtime>::execute(
					staking_precompile_address,
					&alice_input_data,
					None, // target_gas is not necessary right now because consumed none now
					&Context {
						// This context copied from Sacrifice tests, it's not great.
						address: Default::default(),
						caller: Default::default(),
						apparent_value: From::from(0),
					}
				),
				expected_true_result
			);

			// Construct the input data to check if Bob is a collator candidate
			let mut bob_input_data = Vec::<u8>::from([0u8; 36]);
			bob_input_data[0..4].copy_from_slice(&hex_literal::hex!("8545c833"));
			bob_input_data[16..36].copy_from_slice(&CHARLIE);

			// Expected result is an EVM boolean false which is 256 bits long.
			expected_bytes = Vec::from([0u8; 32]);
			let expected_false_result = Some(Ok((ExitSucceed::Returned, expected_bytes, 0)));

			// Assert precompile also reports Bob as not a collator candidate
			assert_eq!(
				ThalesPrecompiles::<Runtime>::execute(
					staking_precompile_address,
					&bob_input_data,
					None,
					&Context {
						// This context copied from Sacrifice tests, it's not great.
						address: Default::default(),
						caller: Default::default(),
						apparent_value: From::from(0),
					}
				),
				expected_false_result
			);
		})
}

#[test]
fn min_nomination_via_precompile() {
	ExtBuilder::default().build().execute_with(|| {
		let staking_precompile_address = H160::from_low_u64_be(2048);

		let mut get_min_nom = Vec::<u8>::from([0u8; 4]);
		get_min_nom[0..4].copy_from_slice(&hex_literal::hex!("c9f593b2"));

		let min_nomination = 5u128 * GLMR;
		let expected_min: U256 = min_nomination.into();
		let mut buffer = [0u8; 32];
		expected_min.to_big_endian(&mut buffer);
		let expected_result = Some(Ok((ExitSucceed::Returned, buffer.to_vec(), 0)));

		assert_eq!(
			ThalesPrecompiles::<Runtime>::execute(
				staking_precompile_address,
				&get_min_nom,
				None,
				&Context {
					// This context copied from Sacrifice tests, it's not great.
					address: Default::default(),
					caller: Default::default(),
					apparent_value: From::from(0),
				}
			),
			expected_result
		);
	});
}
