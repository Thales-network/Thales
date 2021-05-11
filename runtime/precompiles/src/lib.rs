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

#![cfg_attr(not(feature = "std"), no_std)]

mod staking;
use codec::Decode;
use evm::{Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{Precompile, PrecompileSet};
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use sp_core::H160;
use sp_std::convert::TryFrom;
use sp_std::fmt::Debug;
use sp_std::{marker::PhantomData, vec::Vec};
use staking::ParachainStakingWrapper;

use frame_support::traits::Currency;
type BalanceOf<Runtime> = <<Runtime as parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

//TODO Maybe we don't need to / shouldn't be generic over the runtime.
// Pros: Would simplify trait bounds and speed up compile time (maybe not noticeably).
// Cons: Would proclude using this precompile set in mocked Runtimes.
/// The PrecompileSet installed in the Thales runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
#[derive(Debug, Clone, Copy)]
pub struct ThalesPrecompiles<R>(PhantomData<R>);

// The idea here is that we won't have to list the addresses in this file and the chain spec.
// Unfortunately we still have to type it twice in this file.
impl<R: frame_system::Config> ThalesPrecompiles<R>
where
	R::AccountId: From<H160>,
{
	/// Return all addresses that contain precompiles. This can be used to populate dummy code
	/// under the precompile, and potentially in the future to prevent using accounts that have
	/// precompiles at their addresses explicitly using something like SignedExtra.
	#[allow(dead_code)]
	fn used_addresses() -> impl Iterator<Item = R::AccountId> {
		sp_std::vec![1, 2, 3, 4, 5, 6, 7, 8, 1024, 1025, 2048]
			.into_iter()
			.map(|x| hash(x).into())
	}
}

/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither Thales specific
/// 2048-4095 Thales specific precompiles
impl<R> PrecompileSet for ThalesPrecompiles<R>
where
	R::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
	<R::Call as Dispatchable>::Origin: From<Option<R::AccountId>>,
	R: parachain_staking::Config + pallet_evm::Config,
	R::AccountId: From<H160>,
	BalanceOf<R>: TryFrom<sp_core::U256> + Debug,
	R::Call: From<parachain_staking::Call<R>>,
{
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Option<Result<(ExitSucceed, Vec<u8>, u64), ExitError>> {
		match address {
			// Ethereum precompiles :
			a if a == hash(1) => Some(ECRecover::execute(input, target_gas, context)),
			a if a == hash(2) => Some(Sha256::execute(input, target_gas, context)),
			a if a == hash(3) => Some(Ripemd160::execute(input, target_gas, context)),
			a if a == hash(4) => Some(Identity::execute(input, target_gas, context)),
			a if a == hash(5) => Some(Modexp::execute(input, target_gas, context)),
			a if a == hash(6) => Some(Bn128Add::execute(input, target_gas, context)),
			a if a == hash(7) => Some(Bn128Mul::execute(input, target_gas, context)),
			a if a == hash(8) => Some(Bn128Pairing::execute(input, target_gas, context)),
			// Non-Thales specific nor Ethereum precompiles :
			a if a == hash(1024) => Some(Dispatch::<R>::execute(input, target_gas, context)),
			a if a == hash(1025) => Some(Sha3FIPS256::execute(input, target_gas, context)),
			// Thales specific precompiles :
			a if a == hash(2048) => Some(ParachainStakingWrapper::<R>::execute(
				input, target_gas, context,
			)),
			_ => None,
		}
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
