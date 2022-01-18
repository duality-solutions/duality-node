#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{
	generic,
	traits::{IdentifyAccount, Verify},
	MultiAddress, MultiSignature
};

pub use sp_runtime::traits::{BlakeTwo256, Hash as HashT};

/// Opaque, encoded, unchecked extrinsic.
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Public key of account on Substrate-based chains.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Id of account on Substrate-based chains.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// Address of account on Substrate-based chains.
pub type Address = MultiAddress<AccountId, ()>;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Block header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
