// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of THALES.
//
// Copyright (c) 2021 Thales Technologies.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Mocks for the evm-accounts module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, parameter_types, traits::FindAuthor, ConsensusEngineId};
use pallet_evm::EnsureAddressTruncated;
use primitives::Balance;
use sp_core::{crypto::AccountId32, H256};
use sha3::{Digest, Keccak256};
use sp_io::hashing::keccak_256;
use sp_runtime::{testing::Header, traits::IdentityLookup};
use std::collections::BTreeMap;

pub type AccountId = AccountId32;
pub type BlockNumber = u64;

pub const ALICE: AccountId = AccountId32::new([0u8; 32]);
pub const BOB: AccountId = AccountId32::new([1u8; 32]);

mod evm_accounts {
    pub use super::super::*;
}

pub struct AccountInfo {
    pub address: H160,
    pub account_id: AccountId32,
    pub private_key: H256,
}

fn address_build(seed: u8) -> AccountInfo {
    let private_key = H256::from_slice(&[(seed + 1) as u8; 32]); //H256::from_low_u64_be((i + 1) as u64);
    let secret_key = secp256k1::SecretKey::parse_slice(&private_key[..]).unwrap();
    let public_key = &secp256k1::PublicKey::from_secret_key(&secret_key).serialize()[1..65];
    let address = H160::from(H256::from_slice(&Keccak256::digest(public_key)[..]));

    let mut data = [0u8; 32];
    data[0..20].copy_from_slice(&address[..]);

    AccountInfo {
        private_key,
        account_id: AccountId32::from(Into::<[u8; 32]>::into(data)),
        address,
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    // type OnSetCode = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Runtime>;
    type MaxLocks = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1000;
}
impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

/// Fixed gas price of `1`.
pub struct FixedGasPrice;

impl FeeCalculator for FixedGasPrice {
    fn min_gas_price() -> U256 {
        // Gas price is always one token per gas.
        1.into()
    }
}

parameter_types! {
    pub const ChainId: u64 = 9000;
    pub BlockGasLimit: U256 = U256::from(u32::max_value());
}

impl pallet_evm::Config for Runtime {
    type FeeCalculator = FixedGasPrice;
    type GasWeightMapping = ();
    type CallOrigin = EnsureAddressTruncated;
    type WithdrawOrigin = EnsureAddressTruncated;
    type AddressMapping = EvmAddressMapping<Runtime>;
    type Currency = Balances;
    type Event = Event;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type Precompiles = ();
    type ChainId = ChainId;
    type OnChargeTransaction = ();
    type BlockGasLimit = BlockGasLimit;
}

pub struct EthereumFindAuthor;
impl FindAuthor<H160> for EthereumFindAuthor {
    fn find_author<'a, I>(_digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        Some(address_build(0).address)
    }
}

impl pallet_ethereum::Config for Runtime {
    type Event = Event;
    type FindAuthor = EthereumFindAuthor;
    type StateRoot = pallet_ethereum::IntermediateStateRoot;
}

impl Config for Runtime {
    type Event = Event;
    type NativeCurrency = Balances;
    type EvmAddressMapping = EvmAddressMapping<Runtime>;
    type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Storage, Config, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Ethereum: pallet_ethereum::{Module, Call, Storage, Event, ValidateUnsigned},
        EVM: pallet_evm::{Module, Config, Call, Storage, Event<T>},
        EvmAccountsModule: evm_accounts::{Module, Call, Storage, Event<T>},
    }
);

pub struct ExtBuilder();

impl Default for ExtBuilder {
    fn default() -> Self {
        Self()
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: vec![(bob_account_id(), 100000), (ALICE, 1000000000)],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_evm::GenesisConfig {
            accounts: BTreeMap::new(),
        }
        .assimilate_storage::<Runtime>(&mut t)
        .unwrap();

        // pallet_ethereum::GenesisConfig{}.assimilate_storage::<Runtime>(&mut t).unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub fn alice() -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

pub fn bob() -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
}

pub fn bob_account_id() -> AccountId {
    let address = EvmAccountsModule::eth_address(&bob());
    let mut data = [0u8; 32];
    data[0..4].copy_from_slice(b"evm:");
    data[4..24].copy_from_slice(&address[..]);
    AccountId32::from(Into::<[u8; 32]>::into(data))
}
