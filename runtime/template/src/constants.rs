// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

pub mod currency {
    use runtime_primitives::Balance;

	pub const COIN: Balance = 100000000;
	pub const CENT: Balance = COIN / 100;
    pub const MILL: Balance = CENT / 100;
}

pub mod time {
	pub const BLOCK_TIME: u64 = 6; // seconds

	// Time is measured by number of blocks.
    pub const MINUTES: u64 = 60 / BLOCK_TIME;
	pub const HOURS: u64 = MINUTES * 60;
	pub const DAYS: u64 = HOURS * 24;
}

pub mod params {
    use super::{time, currency};

    use frame_support::weights::{Weight, constants::WEIGHT_PER_SECOND};
    use sp_runtime::Perbill;
    use runtime_primitives::Balance;

    /// This determines the average expected block time that we are targeting.
    /// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
    /// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
    /// up by `pallet_aura` to implement `fn slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub const SLOT_DURATION: u64 = time::BLOCK_TIME * 1000; // milliseconds

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
}