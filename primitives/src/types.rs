// Copyright 2018-2019 Parity Technologies (UK) Ltd. and Annie Lai Investments Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Low-level types used by Benson Box.

use codec::{Decode, Encode};
use sr_primitives::{
	generic,
	traits::{BlakeTwo256, Verify},
	AnySignature, DoughnutV0, OpaqueExtrinsic,
};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = AnySignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <Signature as Verify>::Signer;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Asset ID for generic asset module.
pub type AssetId = u32;

/// The runtime supported proof of delegation format.
pub type Doughnut = DoughnutV0;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Index of a transaction in the chain.
pub type Index = u64;

/// A hash of some data used by the chain.
pub type Hash = primitives::H256;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Timestamp = u64;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;

/// The outer `FeeExchange` type. It is versioned to provide flexbility for future iterations
/// while maintaining backward compatability.
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
pub enum FeeExchange {
	/// A V1 FeeExchange
	#[codec(compact)]
	V1(FeeExchangeV1),
}

/// A v1 FeeExchange
/// Signals a fee payment requiring the CENNZX-Spot exchange. It is intended to
/// embed within Benson extrinsic payload.
/// It specifies input asset ID and the max. limit of input asset to pay
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
pub struct FeeExchangeV1 {
	/// The Asset ID to exchange for network fee asset
	#[codec(compact)]
	pub asset_id: AssetId,
	/// The maximum `asset_id` to pay, given the exchange rate
	#[codec(compact)]
	pub max_payment: Balance,
}
