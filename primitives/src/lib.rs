// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{
	generic,
	traits::{IdentifyAccount, Verify},
	MultiAddress, MultiSignature
};

/// Alias to the BLAKE2b 256-bit hashing scheme
pub use sp_runtime::traits::BlakeTwo256 as BlakeTwo256;

/// Alias to the block finality gadget, GRANDPA's AuthorityId
pub use sp_finality_grandpa::AuthorityId as GrandpaId;

/// Opaque, encoded, unchecked extrinsic.
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// An index to a block.
pub type BlockNumber = u32;

/// An instant or duration in time.
pub type Moment = u64;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Public key of account on Substrate-based chains.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Id of account on Substrate-based chains.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Address of account on Substrate-based chains.
pub type Address = MultiAddress<AccountId, ()>;

/// Index of a transaction in the relay chain. 32-bit should be plenty.
pub type Nonce = u32;

/// Balance of an account.
/// 128-bits (or 38 significant decimal figures) will allow for 10m currency (10^7) at a resolution
/// to all for one second's worth of an annualised 50% reward be paid to a unit holder (10^11 unit
/// denomination), or 10^18 total atomic units, to grow at 50%/year for 51 years (10^9 multiplier)
/// for an eventual total of 10^27 units (27 significant decimal figures).
/// We round denomination to 10^12 (12 sdf), and leave the other redundancy at the upper end so
/// that 32 bits may be multiplied with a balance in 128 bits without worrying about overflow.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Block header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// The type used for currency conversion.
///
/// This must only be used as long as the balance type is `u128`.
pub type CurrencyToVote = frame_support::traits::U128CurrencyToVote;
static_assertions::assert_eq_size!(Balance, u128);
