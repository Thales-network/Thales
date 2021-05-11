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
// These clippy lints are disabled because the macro-generated code triggers them.
#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::too_many_arguments)]

use ethereum::Transaction;
use sp_runtime::traits::Block as BlockT;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait TxPoolRuntimeApi {
		fn extrinsic_filter(xt: Vec<<Block as BlockT>::Extrinsic>) -> Vec<Transaction>;
	}
}
