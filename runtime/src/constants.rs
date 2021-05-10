// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of THALES-SUBSTRATE.
//
// Copyright (c) 2021 Thales Network Technologies.
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

/// Money matters.
pub mod currency {
	use primitives::Balance;

	pub const DOLLARS: Balance = 1_000_000_000_000_000_000;
	pub const CENTS: Balance = DOLLARS / 100;
	pub const MILLICENTS: Balance = CENTS / 1_000;
	pub const MICROCENTS: Balance = MILLICENTS / 1_000;

	// Ethereum units
	pub const ETHER: Balance = 1_000_000_000_000_000_000;
	pub const MILLIETHER: Balance = 1_000_000_000_000_000;
	pub const MICROETHER: Balance = 1_000_000_000_000;
	pub const GWEI: Balance = 1_000_000_000;
	pub const MWEI: Balance = 1_000_000;
	pub const KWEI: Balance = 1_000;
	pub const WEI: Balance = 1;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS
	}
}

/// Time and blocks.
pub mod time {
	use primitives::{Moment, BlockNumber};
	// mainnet
	pub const MILLISECS_PER_BLOCK: Moment = 6000;
	// Testnet
//	pub const MILLISECS_PER_BLOCK: Moment = 1000;
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
	// Kusama
	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 1 * HOURS;
	// Mainnet
//	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 4 * HOURS;
	// Testnet
//	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;

	// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}