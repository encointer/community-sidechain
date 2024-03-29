/*
	Copyright 2022 Encointer Association, Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.
*/

//! The Substrate Node Template sgx-runtime for SGX.
//! This is only meant to be used inside an SGX enclave with `#[no_std]`
//!
//! you should assemble your sgx-runtime to be used with your STF here
//! and get all your needed pallets in

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(prelude_import)]
#![feature(structural_match)]
#![feature(core_intrinsics)]
#![feature(derive_eq)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

#[cfg(feature = "evm")]
mod evm;

#[cfg(feature = "evm")]
pub use evm::{
	AddressMapping, EnsureAddressTruncated, EvmCall, FeeCalculator, FixedGasPrice,
	FixedGasWeightMapping, GasWeightMapping, HashedAddressMapping, IntoAddressMapping,
	SubstrateBlockHashMapping, GAS_PER_SECOND, MAXIMUM_BLOCK_WEIGHT, WEIGHT_PER_GAS,
};

use core::convert::{TryFrom, TryInto};
use encointer_primitives::balances::{BalanceType, Demurrage};
use frame_support::weights::ConstantMultiplier;
use frame_system::{EnsureRoot, EnsureSigned, EnsureSignedBy};
use pallet_transaction_payment::CurrencyAdapter;
use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_runtime::{
	create_runtime_str, generic,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, Verify},
	AccountId32, MultiSignature,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

// Re-exports from itp-sgx-runtime-primitives.
pub use itp_sgx_runtime_primitives::{
	constants::SLOT_DURATION,
	types::{
		AccountData, AccountId, Address, Balance, BlockNumber, Hash, Header, Index, Moment,
		Signature,
	},
};

pub use encointer_balances_tx_payment::{AssetBalanceOf, AssetIdOf, BalanceToCommunityBalance};
// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, ord_parameter_types, parameter_types,
	traits::{EitherOfDiverse, KeyOwnerProofSystem, Randomness},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		IdentityFee, Weight,
	},
	StorageValue,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_encointer_balances::Call as EncointerBalancesCall;
pub use pallet_encointer_ceremonies::Call as EncointerCeremoniesCall;
pub use pallet_encointer_communities::Call as EncointerCommunitiesCall;
pub use pallet_parentchain::Call as ParentchainCall;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

/// Block type as expected by this sgx-runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this sgx-runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_asset_tx_payment::ChargeAssetTxPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this sgx-runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsReversedWithSystemFirst,
>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the sgx-runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {

	use sp_runtime::generic;
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = itp_sgx_runtime_primitives::types::Header;
	/// Opaque block type.
	pub type Block = super::Block;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("node-template"),
	impl_name: create_runtime_str!("node-template"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 2400;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in sgx-runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = Header;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the sgx-runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the sgx-runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = AccountData;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	/// The maximum number of consumers allowed on a single account.
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
	type Event = Event;
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

impl pallet_parentchain::Config for Runtime {
	type WeightInfo = ();
}

parameter_types! {
	pub const MomentsPerDay: Moment = 86_400_000; // [ms/d]
	pub const DefaultDemurrage: Demurrage = Demurrage::from_bits(0x0000000000000000000001E3F0A8A973_i128);
	/// 0.000005
	pub const EncointerExistentialDeposit: BalanceType = BalanceType::from_bits(0x0000000000000000000053e2d6238da4_i128);
	pub const MeetupSizeTarget: u64 = 10;
	pub const MeetupMinSize: u64 = 3;
	pub const MeetupNewbieLimitDivider: u64 = 2;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

// Hack to have the same masters on-chain and in the sidechain.
ord_parameter_types! {
	pub const Alice: AccountId32 = AccountId32::new([212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125]);
}
/// Hard coded origin for the pallet's `EnsureOrigin` associated type.
/// Root or Alice (Alice is root in encointer node, The enclave account is root in the sidechain)
pub type EnsureAliceOrRoot =
	EitherOfDiverse<EnsureSignedBy<Alice, AccountId32>, EnsureRoot<AccountId>>;

impl pallet_encointer_scheduler::Config for Runtime {
	type Event = Event;
	type OnCeremonyPhaseChange = pallet_encointer_ceremonies::Pallet<Runtime>;
	type MomentsPerDay = MomentsPerDay;
	type CeremonyMaster = EnsureAliceOrRoot;
	type WeightInfo = ();
}

impl pallet_encointer_communities::Config for Runtime {
	type Event = Event;
	type CommunityMaster = EnsureAliceOrRoot;
	type TrustableForNonDestructiveAction = EnsureSigned<AccountId>;
	type WeightInfo = ();
}

impl pallet_encointer_ceremonies::Config for Runtime {
	type Event = Event;
	type CeremonyMaster = EnsureAliceOrRoot;
	type Public = <MultiSignature as Verify>::Signer;
	type Signature = MultiSignature;
	// Note: in production networks it is advised to use babes randomness source.
	// But we have low security requirements here, so it should be fine.
	type RandomnessSource = pallet_randomness_collective_flip::Pallet<Runtime>;
	type MeetupSizeTarget = MeetupSizeTarget;
	type MeetupMinSize = MeetupMinSize;
	type MeetupNewbieLimitDivider = MeetupNewbieLimitDivider;
	type WeightInfo = ();
}

impl pallet_encointer_balances::Config for Runtime {
	type Event = Event;
	type DefaultDemurrage = DefaultDemurrage;
	type ExistentialDeposit = EncointerExistentialDeposit;
	type WeightInfo = ();
	type CeremonyMaster = EnsureAliceOrRoot;
}

impl pallet_asset_tx_payment::Config for Runtime {
	type Event = Event;
	type Fungibles = pallet_encointer_balances::Pallet<Runtime>;
	type OnChargeAssetTransaction = pallet_asset_tx_payment::FungiblesAdapter<
		encointer_balances_tx_payment::BalanceToCommunityBalance<Runtime>,
		encointer_balances_tx_payment::BurnCredit,
	>;
}

// The plain sgx-runtime without the `evm-pallet`
#[cfg(not(feature = "evm"))]
construct_runtime!(
	pub struct Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 2,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 5,

		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 11,
		AssetTxPayment: pallet_asset_tx_payment::{Pallet, Storage, Event<T>} = 12,

		Parentchain: pallet_parentchain::{Pallet, Call, Storage} = 54,

		EncointerScheduler: pallet_encointer_scheduler::{Pallet, Call, Storage, Config<T>, Event}  = 60,
		EncointerCeremonies: pallet_encointer_ceremonies::{Pallet, Call, Storage, Config<T>, Event<T>} = 61,
		EncointerBalances: pallet_encointer_balances::{Pallet, Call, Storage, Config, Event<T>} = 62,
		EncointerCommunities: pallet_encointer_communities::{Pallet, Call, Storage, Config, Event<T>} = 63,
	}
);

// Runtime constructed with the evm pallet.
//
// We need add the compiler-flag for the whole macro because it does not support
// compiler flags withing the macro.
#[cfg(feature = "evm")]
construct_runtime!(
	pub struct Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 2,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 5,

		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 11,
		AssetTxPayment: pallet_asset_tx_payment::{Pallet, Storage, Event<T>} = 12,

		Parentchain: pallet_parentchain::{Pallet, Call, Storage} = 54,

		EncointerScheduler: pallet_encointer_scheduler::{Pallet, Call, Storage, Config<T>, Event}  = 60,
		EncointerCeremonies: pallet_encointer_ceremonies::{Pallet, Call, Storage, Config<T>, Event<T>} = 61,
		EncointerBalances: pallet_encointer_balances::{Pallet, Call, Storage, Config, Event<T>} = 62,
		EncointerCommunities: pallet_encointer_communities::{Pallet, Call, Storage, Config, Event<T>} = 63,

		Evm: pallet_evm::{Pallet, Call, Storage, Config, Event<T>} = 80,
	}
);

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

}
