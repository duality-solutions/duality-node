// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

pub mod currency {
    use duality_primitives::Balance;

	pub const COIN: Balance = 100000000;
	pub const CENT: Balance = COIN / 100;
    pub const MILL: Balance = CENT / 100;
}

pub mod time {
    use duality_primitives::{Moment, BlockNumber};
	pub const BLOCK_TIME: Moment = 6; // seconds

	// Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60 / BLOCK_TIME as u32;
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
}

pub mod params {
    use super::{time, currency};

    use frame_support::weights::{Weight, constants::WEIGHT_PER_SECOND};
    use sp_runtime::Perbill;
    use duality_primitives::Balance;

    pub mod babe {
        use super::time::*;
        use duality_primitives::BlockNumber;

        /// This determines the average expected block time that we are targeting.
        pub const SLOT_DURATION: u64 = BLOCK_TIME * 1000; // milliseconds

        // 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
        pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

        // NOTE: Currently it is not possible to change the epoch duration after the chain has started.
    	//       Attempting to do so will brick block production.
        pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 1 * DAYS;
    }

    /// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
    /// This is used to limit the maximal weight of a single extrinsic.
    pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);

    /// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
    /// by  Operational  extrinsics.
    pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

    /// This determines the (chain) address prefix for SS58 addresses. 42 is assigned
    /// as the generic prefix for Substrate-based chains.
    pub const SS58_BASE_PREFIX: u8 = 42;

    /// We require that any account contain a certain minimum amount of funds for it be
    /// considered "activated". When an account's balance drops below this threshold,
    /// it will be reaped and any remaining funds will be burned. Reaped accounts cannot
    /// interact with the network in any capacity until the existential deposit has been
    /// replenished.
    pub const EXISTENTIAL_DEPOSIT: Balance = 1 * currency::CENT;

    /// This determines the cost per byte of information conveyed in each transaction
    pub const TRANSACTION_BYTE_FEE: Balance = 1 * currency::MILL;

    /// These determine the cost per key and additional bytes of information stored on-chain
    pub const ITEM_FEE: Balance = currency::CENT;
    pub const STORAGE_BYTE_FEE: Balance = ITEM_FEE / 10;

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
		(items as Balance) * ITEM_FEE + (bytes as Balance) * STORAGE_BYTE_FEE
	}

    static_assertions::const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());
}
