/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

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

#[cfg(feature = "evm")]
use sp_core::{H160, H256, U256};

#[cfg(feature = "evm")]
use std::vec::Vec;

use crate::{
	helpers::ensure_enclave_signer_account, AccountId, KeyPair, Moment, ShardIdentifier, Signature,
	StfError, TrustedOperation,
};
use codec::{Decode, Encode};
use encointer_primitives::{
	balances::{BalanceType, FeeConversionFactorType},
	ceremonies::{
		ClaimOfAttendance, CommunityCeremony, EndorsementTicketsType, InactivityTimeoutType,
		MeetupIndexType, MeetupTimeOffsetType, ProofOfAttendance, ReputationLifetimeType,
	},
	communities::CommunityIdentifier,
	scheduler::CeremonyPhaseType,
};
use frame_support::{ensure, traits::UnfilteredDispatchable};
pub use ita_sgx_runtime::{Balance, Index};
use ita_sgx_runtime::{Runtime, System};
use itp_stf_interface::ExecuteCall;
use itp_storage::storage_value_key;
use itp_types::OpaqueCall;
use itp_utils::stringify::account_id_to_string;
use log::*;
use sp_io::hashing::blake2_256;
use sp_runtime::{traits::Verify, MultiAddress};
use std::{format, prelude::v1::*};

#[cfg(feature = "evm")]
use ita_sgx_runtime::{AddressMapping, HashedAddressMapping};

#[cfg(feature = "evm")]
use crate::evm_helpers::{create_code_hash, evm_create2_address, evm_create_address};

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TrustedCall {
	balance_set_balance(AccountId, AccountId, Balance, Balance),
	balance_transfer(AccountId, AccountId, Balance),
	balance_unshield(AccountId, AccountId, Balance, ShardIdentifier), // (AccountIncognito, BeneficiaryPublicAccount, Amount, Shard)
	balance_shield(AccountId, AccountId, Balance), // (Root, AccountIncognito, Amount)
	encointer_balance_transfer(AccountId, AccountId, CommunityIdentifier, BalanceType),
	encointer_set_fee_conversion_factor(AccountId, FeeConversionFactorType),
	encointer_transfer_all(AccountId, AccountId, CommunityIdentifier),
	ceremonies_register_participant(
		AccountId,
		CommunityIdentifier,
		Option<ProofOfAttendance<Signature, AccountId>>,
	),
	ceremonies_upgrade_registration(
		AccountId,
		CommunityIdentifier,
		ProofOfAttendance<Signature, AccountId>,
	),
	ceremonies_unregister_participant(AccountId, CommunityIdentifier, Option<CommunityCeremony>),
	ceremonies_attest_attendees(AccountId, CommunityIdentifier, u32, Vec<AccountId>),
	ceremonies_attest_claims(AccountId, Vec<ClaimOfAttendance<Signature, AccountId, Moment>>),
	ceremonies_endorse_newcomer(AccountId, CommunityIdentifier, AccountId),
	ceremonies_claim_rewards(AccountId, CommunityIdentifier, Option<MeetupIndexType>),
	ceremonies_set_inactivity_timeout(AccountId, InactivityTimeoutType),
	ceremonies_set_endorsement_tickets_per_bootstrapper(AccountId, EndorsementTicketsType),
	ceremonies_set_endorsement_tickets_per_reputable(AccountId, EndorsementTicketsType),
	ceremonies_set_reputation_lifetime(AccountId, ReputationLifetimeType),
	ceremonies_set_meetup_time_offset(AccountId, MeetupTimeOffsetType),
	ceremonies_set_time_tolerance(AccountId, Moment),
	ceremonies_set_location_tolerance(AccountId, u32),
	ceremonies_purge_community_ceremony(AccountId, CommunityCeremony),
	#[cfg(feature = "evm")]
	evm_withdraw(AccountId, H160, Balance), // (Origin, Address EVM Account, Value)
	// (Origin, Source, Target, Input, Value, Gas limit, Max fee per gas, Max priority fee per gas, Nonce, Access list)
	#[cfg(feature = "evm")]
	evm_call(
		AccountId,
		H160,
		H160,
		Vec<u8>,
		U256,
		u64,
		U256,
		Option<U256>,
		Option<U256>,
		Vec<(H160, Vec<H256>)>,
	),
	// (Origin, Source, Init, Value, Gas limit, Max fee per gas, Max priority fee per gas, Nonce, Access list)
	#[cfg(feature = "evm")]
	evm_create(
		AccountId,
		H160,
		Vec<u8>,
		U256,
		u64,
		U256,
		Option<U256>,
		Option<U256>,
		Vec<(H160, Vec<H256>)>,
	),
	// (Origin, Source, Init, Salt, Value, Gas limit, Max fee per gas, Max priority fee per gas, Nonce, Access list)
	#[cfg(feature = "evm")]
	evm_create2(
		AccountId,
		H160,
		Vec<u8>,
		H256,
		U256,
		u64,
		U256,
		Option<U256>,
		Option<U256>,
		Vec<(H160, Vec<H256>)>,
	),
}

impl TrustedCall {
	pub fn sender_account(&self) -> &AccountId {
		match self {
			TrustedCall::balance_set_balance(sender_account, ..) => sender_account,
			TrustedCall::balance_transfer(sender_account, ..) => sender_account,
			TrustedCall::balance_unshield(sender_account, ..) => sender_account,
			TrustedCall::balance_shield(sender_account, ..) => sender_account,
			TrustedCall::encointer_balance_transfer(sender_account, ..) => sender_account,
			TrustedCall::encointer_set_fee_conversion_factor(sender_account, ..) => sender_account,
			TrustedCall::encointer_transfer_all(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_register_participant(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_upgrade_registration(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_unregister_participant(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_attest_attendees(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_attest_claims(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_endorse_newcomer(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_claim_rewards(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_set_inactivity_timeout(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_set_endorsement_tickets_per_bootstrapper(
				sender_account,
				..,
			) => sender_account,
			TrustedCall::ceremonies_set_endorsement_tickets_per_reputable(sender_account, ..) =>
				sender_account,
			TrustedCall::ceremonies_set_reputation_lifetime(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_set_meetup_time_offset(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_set_time_tolerance(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_set_location_tolerance(sender_account, ..) => sender_account,
			TrustedCall::ceremonies_purge_community_ceremony(sender_account, ..) => sender_account,
			#[cfg(feature = "evm")]
			TrustedCall::evm_withdraw(sender_account, ..) => sender_account,
			#[cfg(feature = "evm")]
			TrustedCall::evm_call(sender_account, ..) => sender_account,
			#[cfg(feature = "evm")]
			TrustedCall::evm_create(sender_account, ..) => sender_account,
			#[cfg(feature = "evm")]
			TrustedCall::evm_create2(sender_account, ..) => sender_account,
		}
	}

	pub fn sign(
		&self,
		pair: &KeyPair,
		nonce: Index,
		mrenclave: &[u8; 32],
		shard: &ShardIdentifier,
	) -> TrustedCallSigned {
		let mut payload = self.encode();
		payload.append(&mut nonce.encode());
		payload.append(&mut mrenclave.encode());
		payload.append(&mut shard.encode());

		TrustedCallSigned { call: self.clone(), nonce, signature: pair.sign(payload.as_slice()) }
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct TrustedCallSigned {
	pub call: TrustedCall,
	pub nonce: Index,
	pub signature: Signature,
}

impl TrustedCallSigned {
	pub fn new(call: TrustedCall, nonce: Index, signature: Signature) -> Self {
		TrustedCallSigned { call, nonce, signature }
	}

	pub fn verify_signature(&self, mrenclave: &[u8; 32], shard: &ShardIdentifier) -> bool {
		let mut payload = self.call.encode();
		payload.append(&mut self.nonce.encode());
		payload.append(&mut mrenclave.encode());
		payload.append(&mut shard.encode());
		self.signature.verify(payload.as_slice(), self.call.sender_account())
	}

	pub fn into_trusted_operation(self, direct: bool) -> TrustedOperation {
		match direct {
			true => TrustedOperation::direct_call(self),
			false => TrustedOperation::indirect_call(self),
		}
	}
}

// TODO: #91 signed return value
/*
pub struct TrustedReturnValue<T> {
	pub value: T,
	pub signer: AccountId
}

impl TrustedReturnValue
*/

impl ExecuteCall for TrustedCallSigned {
	type Error = StfError;

	fn execute(
		self,
		calls: &mut Vec<OpaqueCall>,
		unshield_funds_fn: [u8; 2],
	) -> Result<(), Self::Error> {
		let sender = self.call.sender_account().clone();
		let call_hash = blake2_256(&self.call.encode());
		ensure!(
			self.nonce == System::account_nonce(&sender),
			Self::Error::InvalidNonce(self.nonce)
		);
		match self.call {
			TrustedCall::balance_set_balance(root, who, free_balance, reserved_balance) => {
				ensure!(is_root::<Runtime, AccountId>(&root), Self::Error::MissingPrivileges(root));
				debug!(
					"balance_set_balance({}, {}, {})",
					account_id_to_string(&who),
					free_balance,
					reserved_balance
				);
				ita_sgx_runtime::BalancesCall::<Runtime>::set_balance {
					who: MultiAddress::Id(who),
					new_free: free_balance,
					new_reserved: reserved_balance,
				}
				.dispatch_bypass_filter(ita_sgx_runtime::Origin::root())
				.map_err(|e| {
					Self::Error::Dispatch(format!("Balance Set Balance error: {:?}", e.error))
				})?;
				Ok(())
			},
			TrustedCall::balance_transfer(from, to, value) => {
				let origin = ita_sgx_runtime::Origin::signed(from.clone());
				debug!(
					"balance_transfer({}, {}, {})",
					account_id_to_string(&from),
					account_id_to_string(&to),
					value
				);
				ita_sgx_runtime::BalancesCall::<Runtime>::transfer {
					dest: MultiAddress::Id(to),
					value,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!("Balance Transfer error: {:?}", e.error))
				})?;
				Ok(())
			},
			TrustedCall::balance_unshield(account_incognito, beneficiary, value, shard) => {
				debug!(
					"balance_unshield({}, {}, {}, {})",
					account_id_to_string(&account_incognito),
					account_id_to_string(&beneficiary),
					value,
					shard
				);
				unshield_funds(account_incognito, value)?;
				calls.push(OpaqueCall::from_tuple(&(
					unshield_funds_fn,
					beneficiary,
					value,
					shard,
					call_hash,
				)));
				Ok(())
			},
			TrustedCall::balance_shield(enclave_account, who, value) => {
				ensure_enclave_signer_account(&enclave_account)?;
				debug!("balance_shield({}, {})", account_id_to_string(&who), value);
				shield_funds(who, value)?;
				Ok(())
			},
			TrustedCall::encointer_balance_transfer(from, to, community_id, value) => {
				let origin = ita_sgx_runtime::Origin::signed(from.clone());
				debug!(
					"encointer_balance_transfer({}, {}, {}, {})",
					account_id_to_string(&from),
					account_id_to_string(&to),
					community_id,
					value
				);
				ita_sgx_runtime::EncointerBalancesCall::<Runtime>::transfer {
					dest: AccountId::from(to),
					community_id,
					amount: value,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Encointer Balance Transfer error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::encointer_set_fee_conversion_factor(who, fee_conversion_factor) => {
				let origin = ita_sgx_runtime::Origin::signed(who.clone());
				debug!(
					"encointer_set_fee_conversion_factor({}, {})",
					account_id_to_string(&who),
					fee_conversion_factor
				);
				ita_sgx_runtime::EncointerBalancesCall::<Runtime>::set_fee_conversion_factor {
					fee_conversion_factor,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Encointer Balance set fee conversion error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::encointer_transfer_all(from, to, community_id) => {
				let origin = ita_sgx_runtime::Origin::signed(from.clone());
				debug!(
					"encointer_transfer_all({}, {}, {})",
					account_id_to_string(&from),
					account_id_to_string(&to),
					community_id
				);
				ita_sgx_runtime::EncointerBalancesCall::<Runtime>::transfer_all {
					dest: AccountId::from(to),
					cid: community_id,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Encointer Balance transfer all error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_register_participant(who, cid, proof) => {
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				if pallet_encointer_scheduler::Pallet::<ita_sgx_runtime::Runtime>::current_phase()
					== CeremonyPhaseType::Assigning
				{
					return Err(Self::Error::Dispatch(
						"registering participants can only be done during registering or attesting phase"
							.to_string(),
					))
				}

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::register_participant {
					cid,
					proof,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies register participant error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_upgrade_registration(who, cid, proof) => {
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				if pallet_encointer_scheduler::Pallet::<ita_sgx_runtime::Runtime>::current_phase()
					== CeremonyPhaseType::Assigning
				{
					return Err(Self::Error::Dispatch(
						"upgrading registration can only be done during registering or attesting phase"
							.to_string(),
					))
				}

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::upgrade_registration {
					cid,
					proof: proof.clone(),
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies upgrade registration error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_unregister_participant(
				who,
				cid,
				maybe_reputation_community_ceremony,
			) => {
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				if pallet_encointer_scheduler::Pallet::<ita_sgx_runtime::Runtime>::current_phase()
					== CeremonyPhaseType::Assigning
				{
					return Err(Self::Error::Dispatch(
						"unregistering participant can only be done during registering or attesting phase"
							.to_string(),
					))
				}

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::unregister_participant {
					cid,
					maybe_reputation_community_ceremony,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies unregister participant error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_attest_attendees(
				who,
				cid,
				number_of_participants_vote,
				attestations,
			) => {
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				if pallet_encointer_scheduler::Pallet::<ita_sgx_runtime::Runtime>::current_phase()
					!= CeremonyPhaseType::Attesting
				{
					return Err(Self::Error::Dispatch(
						"attendees attestation can only be done during attesting phase".to_string(),
					))
				}

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::attest_attendees {
					cid,
					number_of_participants_vote,
					attestations,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies attendees attestation error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_attest_claims(who, claims) => {
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				if pallet_encointer_scheduler::Pallet::<ita_sgx_runtime::Runtime>::current_phase()
					!= CeremonyPhaseType::Attesting
				{
					return Err(Self::Error::Dispatch(
						"claims attestation can only be done during attesting phase".to_string(),
					))
				}

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::attest_claims { claims }
					.dispatch_bypass_filter(origin)
					.map_err(|e| {
						Self::Error::Dispatch(format!(
							"Ceremonies claims attestation error: {:?}",
							e.error
						))
					})?;
				Ok(())
			},
			TrustedCall::ceremonies_endorse_newcomer(who, cid, newbie) => {
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::endorse_newcomer {
					cid,
					newbie,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies endorse newcomer error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_claim_rewards(who, cid, maybe_meetup_index) => {
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				if pallet_encointer_scheduler::Pallet::<ita_sgx_runtime::Runtime>::current_phase()
					== CeremonyPhaseType::Assigning
				{
					return Err(Self::Error::Dispatch(
						"claiming rewards can not be done during assigning phase".to_string(),
					))
				}

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::claim_rewards {
					cid,
					maybe_meetup_index,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!("Ceremonies claim rewards error: {:?}", e.error))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_set_inactivity_timeout(who, inactivity_timeout) => {
				//Master check
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::set_inactivity_timeout {
					inactivity_timeout,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies set inactivity timeout error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_set_endorsement_tickets_per_bootstrapper(
				who,
				endorsement_tickets_per_bootstrapper,
			) => {
				//Master check
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::set_endorsement_tickets_per_bootstrapper {
					endorsement_tickets_per_bootstrapper,
				}
					.dispatch_bypass_filter(origin)
					.map_err(|e| {
						Self::Error::Dispatch(format!(
							"Ceremonies set endorsement ticket per bootstrapper error: {:?}",
							e.error
						))
					})?;
				Ok(())
			},
			TrustedCall::ceremonies_set_endorsement_tickets_per_reputable(
				who,
				endorsement_tickets_per_reputable,
			) => {
				//Master check
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::set_endorsement_tickets_per_reputable {
					endorsement_tickets_per_reputable,
				}
					.dispatch_bypass_filter(origin)
					.map_err(|e| {
						Self::Error::Dispatch(format!(
							"Ceremonies set endorsement ticket per reputable error: {:?}",
							e.error
						))
					})?;
				Ok(())
			},
			TrustedCall::ceremonies_set_reputation_lifetime(who, reputation_lifetime) => {
				//Master check
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::set_reputation_lifetime {
					reputation_lifetime,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies set reputation lifetime error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_set_meetup_time_offset(who, meetup_time_offset) => {
				//Check Master
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				if pallet_encointer_scheduler::Pallet::<ita_sgx_runtime::Runtime>::current_phase()
					== CeremonyPhaseType::Registering
				{
					return Err(Self::Error::Dispatch(
						"setting meetup time offset can not be done during registering phase"
							.to_string(),
					))
				}

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::set_meetup_time_offset {
					meetup_time_offset,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies set meetup time offset error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_set_time_tolerance(who, time_tolerance) => {
				//Master check
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::set_time_tolerance {
					time_tolerance,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies set time tolerance error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_set_location_tolerance(who, location_tolerance) => {
				//Master check
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::set_location_tolerance {
					location_tolerance,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies set location tolerance error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			TrustedCall::ceremonies_purge_community_ceremony(who, community_ceremony) => {
				//Master check
				let origin = ita_sgx_runtime::Origin::signed(who.clone());

				ita_sgx_runtime::EncointerCeremoniesCall::<Runtime>::purge_community_ceremony {
					community_ceremony,
				}
				.dispatch_bypass_filter(origin)
				.map_err(|e| {
					Self::Error::Dispatch(format!(
						"Ceremonies purge community ceremony error: {:?}",
						e.error
					))
				})?;
				Ok(())
			},
			#[cfg(feature = "evm")]
			TrustedCall::evm_withdraw(from, address, value) => {
				debug!("evm_withdraw({}, {}, {})", account_id_to_string(&from), address, value);
				ita_sgx_runtime::EvmCall::<Runtime>::withdraw { address, value }
					.dispatch_bypass_filter(ita_sgx_runtime::Origin::signed(from))
					.map_err(|e| {
						Self::Error::Dispatch(format!("Evm Withdraw error: {:?}", e.error))
					})?;
				Ok(())
			},
			#[cfg(feature = "evm")]
			TrustedCall::evm_call(
				from,
				source,
				target,
				input,
				value,
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list,
			) => {
				debug!(
					"evm_call(from: {}, source: {}, target: {})",
					account_id_to_string(&from),
					source,
					target
				);
				ita_sgx_runtime::EvmCall::<Runtime>::call {
					source,
					target,
					input,
					value,
					gas_limit,
					max_fee_per_gas,
					max_priority_fee_per_gas,
					nonce,
					access_list,
				}
				.dispatch_bypass_filter(ita_sgx_runtime::Origin::signed(from))
				.map_err(|e| Self::Error::Dispatch(format!("Evm Call error: {:?}", e.error)))?;
				Ok(())
			},
			#[cfg(feature = "evm")]
			TrustedCall::evm_create(
				from,
				source,
				init,
				value,
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list,
			) => {
				debug!(
					"evm_create(from: {}, source: {}, value: {})",
					account_id_to_string(&from),
					source,
					value
				);
				let nonce_evm_account =
					System::account_nonce(&HashedAddressMapping::into_account_id(source));
				ita_sgx_runtime::EvmCall::<Runtime>::create {
					source,
					init,
					value,
					gas_limit,
					max_fee_per_gas,
					max_priority_fee_per_gas,
					nonce,
					access_list,
				}
				.dispatch_bypass_filter(ita_sgx_runtime::Origin::signed(from))
				.map_err(|e| Self::Error::Dispatch(format!("Evm Create error: {:?}", e.error)))?;
				let contract_address = evm_create_address(source, nonce_evm_account);
				info!("Trying to create evm contract with address {:?}", contract_address);
				Ok(())
			},
			#[cfg(feature = "evm")]
			TrustedCall::evm_create2(
				from,
				source,
				init,
				salt,
				value,
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list,
			) => {
				debug!(
					"evm_create2(from: {}, source: {}, value: {})",
					account_id_to_string(&from),
					source,
					value
				);
				let code_hash = create_code_hash(&init);
				ita_sgx_runtime::EvmCall::<Runtime>::create2 {
					source,
					init,
					salt,
					value,
					gas_limit,
					max_fee_per_gas,
					max_priority_fee_per_gas,
					nonce,
					access_list,
				}
				.dispatch_bypass_filter(ita_sgx_runtime::Origin::signed(from))
				.map_err(|e| Self::Error::Dispatch(format!("Evm Create2 error: {:?}", e.error)))?;
				let contract_address = evm_create2_address(source, salt, code_hash);
				info!("Trying to create evm contract with address {:?}", contract_address);
				Ok(())
			},
		}?;
		System::inc_account_nonce(&sender);
		Ok(())
	}

	fn get_storage_hashes_to_update(self) -> Vec<Vec<u8>> {
		let mut key_hashes = Vec::new();
		match self.call {
			TrustedCall::balance_set_balance(_, _, _, _) => debug!("No storage updates needed..."),
			TrustedCall::balance_transfer(_, _, _) => debug!("No storage updates needed..."),
			TrustedCall::balance_unshield(_, _, _, _) => debug!("No storage updates needed..."),
			TrustedCall::balance_shield(_, _, _) => debug!("No storage updates needed..."),
			TrustedCall::encointer_balance_transfer(_, _, _, _) =>
				debug!("No storage updates needed..."),
			TrustedCall::encointer_set_fee_conversion_factor(_, _) =>
				debug!("No storage updates needed..."),
			TrustedCall::encointer_transfer_all(_, _, _) => debug!("No storage updates needed..."),
			TrustedCall::ceremonies_set_inactivity_timeout(_, _) =>
				debug!("No storage updates needed..."),
			TrustedCall::ceremonies_set_endorsement_tickets_per_bootstrapper(_, _) =>
				debug!("No storage updates needed..."),
			TrustedCall::ceremonies_set_endorsement_tickets_per_reputable(_, _) =>
				debug!("No storage updates needed..."),
			TrustedCall::ceremonies_set_reputation_lifetime(_, _) =>
				debug!("No storage updates needed..."),
			TrustedCall::ceremonies_set_time_tolerance(_, _) =>
				debug!("No storage updates needed..."),
			TrustedCall::ceremonies_set_location_tolerance(_, _) =>
				debug!("No storage updates needed..."),
			TrustedCall::ceremonies_purge_community_ceremony(_, _) =>
				debug!("No storage updates needed..."),
			TrustedCall::ceremonies_register_participant(_, _, _)
			| TrustedCall::ceremonies_upgrade_registration(_, _, _)
			| TrustedCall::ceremonies_unregister_participant(_, _, _)
			| TrustedCall::ceremonies_attest_claims(_, _) => {
				key_hashes.push(storage_value_key("EncointerScheduler", "CurrentPhase"));
				key_hashes.push(storage_value_key("EncointerScheduler", "CurrentCeremonyIndex"));
				key_hashes.push(storage_value_key("EncointerCommunities", "CommunityIdentifiers"));
			},
			TrustedCall::ceremonies_attest_attendees(_, _, _, _)
			| TrustedCall::ceremonies_claim_rewards(_, _, _) => {
				key_hashes.push(storage_value_key("EncointerScheduler", "CurrentPhase"));
				key_hashes.push(storage_value_key("EncointerCommunities", "CommunityIdentifiers"));
			},
			TrustedCall::ceremonies_set_meetup_time_offset(_, _) => {
				key_hashes.push(storage_value_key("EncointerScheduler", "CurrentPhase"));
			},
			TrustedCall::ceremonies_endorse_newcomer(_, _, _) => {
				key_hashes.push(storage_value_key("EncointerScheduler", "CurrentPhase"));
				key_hashes.push(storage_value_key("EncointerScheduler", "CurrentCeremonyIndex"));
				key_hashes.push(storage_value_key("EncointerCommunities", "CommunityIdentifiers"));
				key_hashes.push(storage_value_key("EncointerCommunities", "Bootstrappers"));
			},
			#[cfg(feature = "evm")]
			_ => debug!("No storage updates needed..."),
		};
		key_hashes
	}
}

fn unshield_funds(account: AccountId, amount: u128) -> Result<(), StfError> {
	let account_info = System::account(&account);
	if account_info.data.free < amount {
		return Err(StfError::MissingFunds)
	}

	ita_sgx_runtime::BalancesCall::<Runtime>::set_balance {
		who: MultiAddress::Id(account),
		new_free: account_info.data.free - amount,
		new_reserved: account_info.data.reserved,
	}
	.dispatch_bypass_filter(ita_sgx_runtime::Origin::root())
	.map_err(|e| StfError::Dispatch(format!("Unshield funds error: {:?}", e.error)))?;
	Ok(())
}

fn shield_funds(account: AccountId, amount: u128) -> Result<(), StfError> {
	let account_info = System::account(&account);
	ita_sgx_runtime::BalancesCall::<Runtime>::set_balance {
		who: MultiAddress::Id(account),
		new_free: account_info.data.free + amount,
		new_reserved: account_info.data.reserved,
	}
	.dispatch_bypass_filter(ita_sgx_runtime::Origin::root())
	.map_err(|e| StfError::Dispatch(format!("Shield funds error: {:?}", e.error)))?;

	Ok(())
}

fn is_root<Runtime, AccountId>(account: &AccountId) -> bool
where
	Runtime: frame_system::Config<AccountId = AccountId> + pallet_sudo::Config,
	AccountId: PartialEq,
{
	pallet_sudo::Pallet::<Runtime>::key().map_or(false, |k| account == &k)
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_keyring::AccountKeyring;

	#[test]
	fn verify_signature_works() {
		let nonce = 21;
		let mrenclave = [0u8; 32];
		let shard = ShardIdentifier::default();

		let call = TrustedCall::balance_set_balance(
			AccountKeyring::Alice.public().into(),
			AccountKeyring::Alice.public().into(),
			42,
			42,
		);
		let signed_call =
			call.sign(&KeyPair::Sr25519(AccountKeyring::Alice.pair()), nonce, &mrenclave, &shard);

		assert!(signed_call.verify_signature(&mrenclave, &shard));
	}
}
