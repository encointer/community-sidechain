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

use crate::{AccountId, KeyPair, Signature};
use codec::{Decode, Encode};
use encointer_primitives::{
	ceremonies::{AttestationIndexType, ParticipantIndexType},
	communities::CommunityIdentifier,
	scheduler::CeremonyIndexType,
};
use ita_sgx_runtime::System;
use itp_stf_interface::ExecuteGetter;
use itp_utils::stringify::account_id_to_string;
use log::*;
use sp_runtime::traits::Verify;
use std::prelude::v1::*;

#[cfg(feature = "evm")]
use ita_sgx_runtime::{AddressMapping, HashedAddressMapping};

#[cfg(feature = "evm")]
use crate::evm_helpers::{get_evm_account, get_evm_account_codes, get_evm_account_storages};

use crate::encointer_helpers::is_ceremony_master;

use itp_storage::{storage_map_key, storage_value_key, StorageHasher};
#[cfg(feature = "evm")]
use sp_core::{H160, H256};

pub type EncointerCeremonies = pallet_encointer_ceremonies::Pallet<ita_sgx_runtime::Runtime>;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Getter {
	public(PublicGetter),
	trusted(TrustedGetterSigned),
}

impl From<PublicGetter> for Getter {
	fn from(item: PublicGetter) -> Self {
		Getter::public(item)
	}
}

impl From<TrustedGetterSigned> for Getter {
	fn from(item: TrustedGetterSigned) -> Self {
		Getter::trusted(item)
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum PublicGetter {
	some_value,
	encointer_total_issuance(CommunityIdentifier),
	ceremonies_assignment_counts(CommunityIdentifier, CeremonyIndexType),
	ceremonies_attestation_count(CommunityIdentifier, CeremonyIndexType),
	ceremonies_meetup_count(CommunityIdentifier, CeremonyIndexType),
	ceremonies_meetup_time_offset(),
	ceremonies_registered_bootstrappers_count(CommunityIdentifier, CeremonyIndexType),
	ceremonies_registered_endorsees_count(CommunityIdentifier, CeremonyIndexType),
	ceremonies_registered_newbies_count(CommunityIdentifier, CeremonyIndexType),
	ceremonies_registered_reputables_count(CommunityIdentifier, CeremonyIndexType),
	ceremonies_reward(CommunityIdentifier),
	/*
	   encointer_scheduler_state(CommunityIdentifier),
	   ceremonies_location_tolerance(CommunityIdentifier),
	   ceremonies_time_tolerance(CommunityIdentifier),
	   ceremonies_issued_rewards(CommunityIdentifier, CeremonyIndexType, MeetupIndexType),
	   ceremonies_inactivity_counters(CommunityIdentifier, CeremonyIndexType),
	   ceremonies_inactivity_timeout(CommunityIdentifier),
	   ceremonies_endorsement_tickets_per_bootstrapper(CommunityIdentifier),
	   ceremonies_endorsement_tickets_per_reputable(CommunityIdentifier),
	   ceremonies_reputation_lifetime(CommunityIdentifier),
	*/
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TrustedGetter {
	free_balance(AccountId),
	reserved_balance(AccountId),
	nonce(AccountId),
	encointer_balance(AccountId, CommunityIdentifier),
	ceremonies_aggregated_account_data(AccountId, CommunityIdentifier, AccountId),
	ceremonies_assignments(AccountId, CommunityIdentifier, CeremonyIndexType),
	//ceremonies_reputations(AccountId, CommunityIdentifier),
	//ceremonies_participant_reputation(AccountId, CommunityIdentifier, CeremonyIndexType),
	ceremonies_meetup_participant_count_vote(
		AccountId,
		CommunityIdentifier,
		CeremonyIndexType,
		AccountId,
	),
	ceremonies_participant_attestees(
		AccountId,
		CommunityIdentifier,
		CeremonyIndexType,
		AttestationIndexType,
	),
	ceremonies_participant_attestation_index(
		AccountId,
		CommunityIdentifier,
		CeremonyIndexType,
		AccountId,
	),
	ceremonies_registered_bootstrapper(
		AccountId,
		CommunityIdentifier,
		CeremonyIndexType,
		ParticipantIndexType,
	),
	ceremonies_registered_bootstrappers(AccountId, CommunityIdentifier, CeremonyIndexType),
	ceremonies_registered_reputable(
		AccountId,
		CommunityIdentifier,
		CeremonyIndexType,
		ParticipantIndexType,
	),
	ceremonies_registered_reputables(AccountId, CommunityIdentifier, CeremonyIndexType),
	ceremonies_registered_endorsee(
		AccountId,
		CommunityIdentifier,
		CeremonyIndexType,
		ParticipantIndexType,
	),
	ceremonies_registered_endorsees(AccountId, CommunityIdentifier, CeremonyIndexType),
	ceremonies_registered_newbie(
		AccountId,
		CommunityIdentifier,
		CeremonyIndexType,
		ParticipantIndexType,
	),
	ceremonies_registered_newbies(AccountId, CommunityIdentifier, CeremonyIndexType),

	//bootstrapper_newbie_tickets(CommunityIdentifier, CeremonyIndexType,AccountId),
	//reputable_newbie_tickets(CommunityIdentifier, CeremonyIndexType,AccountId),
	//ceremonies_reputations(AccountId, CommunityIdentifier),
	#[cfg(feature = "evm")]
	evm_nonce(AccountId),
	#[cfg(feature = "evm")]
	evm_account_codes(AccountId, H160),
	#[cfg(feature = "evm")]
	evm_account_storages(AccountId, H160, H256),
}

impl TrustedGetter {
	pub fn sender_account(&self) -> &AccountId {
		match self {
			TrustedGetter::free_balance(sender_account) => sender_account,
			TrustedGetter::reserved_balance(sender_account) => sender_account,
			TrustedGetter::nonce(sender_account) => sender_account,
			TrustedGetter::encointer_balance(sender_account, _) => sender_account,
			TrustedGetter::ceremonies_aggregated_account_data(sender_account, _, _) =>
				sender_account,
			TrustedGetter::ceremonies_assignments(sender_account, _, _) => sender_account,
			TrustedGetter::ceremonies_meetup_participant_count_vote(sender_account, _, _, _) =>
				sender_account,
			TrustedGetter::ceremonies_participant_attestees(sender_account, _, _, _) =>
				sender_account,
			TrustedGetter::ceremonies_participant_attestation_index(sender_account, _, _, _) =>
				sender_account,
			TrustedGetter::ceremonies_registered_bootstrapper(sender_account, _, _, _) =>
				sender_account,
			TrustedGetter::ceremonies_registered_bootstrappers(sender_account, _, _) =>
				sender_account,
			TrustedGetter::ceremonies_registered_reputable(sender_account, _, _, _) =>
				sender_account,
			TrustedGetter::ceremonies_registered_reputables(sender_account, _, _) => sender_account,
			TrustedGetter::ceremonies_registered_endorsee(sender_account, _, _, _) =>
				sender_account,
			TrustedGetter::ceremonies_registered_endorsees(sender_account, _, _) => sender_account,
			TrustedGetter::ceremonies_registered_newbie(sender_account, _, _, _) => sender_account,
			TrustedGetter::ceremonies_registered_newbies(sender_account, _, _) => sender_account,
			#[cfg(feature = "evm")]
			TrustedGetter::evm_nonce(sender_account) => sender_account,
			#[cfg(feature = "evm")]
			TrustedGetter::evm_account_codes(sender_account, _) => sender_account,
			#[cfg(feature = "evm")]
			TrustedGetter::evm_account_storages(sender_account, ..) => sender_account,
		}
	}

	pub fn sign(&self, pair: &KeyPair) -> TrustedGetterSigned {
		let signature = pair.sign(self.encode().as_slice());
		TrustedGetterSigned { getter: self.clone(), signature }
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct TrustedGetterSigned {
	pub getter: TrustedGetter,
	pub signature: Signature,
}

impl TrustedGetterSigned {
	pub fn new(getter: TrustedGetter, signature: Signature) -> Self {
		TrustedGetterSigned { getter, signature }
	}

	pub fn verify_signature(&self) -> bool {
		self.signature
			.verify(self.getter.encode().as_slice(), self.getter.sender_account())
	}
}

impl ExecuteGetter for Getter {
	fn execute(self) -> Option<Vec<u8>> {
		match self {
			Getter::trusted(g) => g.execute(),
			Getter::public(g) => g.execute(),
		}
	}

	fn get_storage_hashes_to_update(self) -> Vec<Vec<u8>> {
		match self {
			Getter::trusted(g) => g.get_storage_hashes_to_update(),
			Getter::public(g) => g.get_storage_hashes_to_update(),
		}
	}
}

impl ExecuteGetter for TrustedGetterSigned {
	fn execute(self) -> Option<Vec<u8>> {
		match self.getter {
			TrustedGetter::free_balance(who) => {
				let info = System::account(&who);
				debug!("TrustedGetter free_balance");
				debug!("AccountInfo for {} is {:?}", account_id_to_string(&who), info);
				debug!("Account free balance is {}", info.data.free);
				Some(info.data.free.encode())
			},

			TrustedGetter::reserved_balance(who) => {
				let info = System::account(&who);
				debug!("TrustedGetter reserved_balance");
				debug!("AccountInfo for {} is {:?}", account_id_to_string(&who), info);
				debug!("Account reserved balance is {}", info.data.reserved);
				Some(info.data.reserved.encode())
			},
			TrustedGetter::nonce(who) => {
				let nonce = System::account_nonce(&who);
				debug!("TrustedGetter nonce");
				debug!("Account nonce is {}", nonce);
				Some(nonce.encode())
			},
			TrustedGetter::encointer_balance(who, community_id) => {
				let balance =
					pallet_encointer_balances::Pallet::<ita_sgx_runtime::Runtime>::balance(
						community_id,
						&who,
					);
				debug!("TrustedGetter encointer_balance");
				Some(balance.encode())
			},
			/*
			TrustedGetter::ceremonies_attestations(who, community_id) => {
				let ceremony_index = pallet_encointer_scheduler::Pallet::<
					ita_sgx_runtime::Runtime,
				>::current_ceremony_index();
				let attestation_index =
					EncointerCeremonies::attestation_index((community_id, ceremony_index), who);
				let attestations = EncointerCeremonies::attestation_registry(
					(community_id, ceremony_index),
					attestation_index,
				);
				Some(attestations.encode())
			},
			 */
			TrustedGetter::ceremonies_aggregated_account_data(who, community_id, account_id) => {
				debug!("TrustedGetter ceremonies_aggregated_account_data");
				//Todo Master or Me?
				if !is_ceremony_master(who) {
					debug!("TrustedGetter ceremonies_aggregated_account_data, return: No master");
					return None
				}
				let aggregated_account_data =
					EncointerCeremonies::get_aggregated_account_data(community_id, &account_id);
				Some(aggregated_account_data.encode())
			},
			TrustedGetter::ceremonies_assignments(who, community_id, ceremony_index) => {
				debug!("TrustedGetter ceremonies_assignments");
				// Block getter of confidential data if it is not the CeremonyMaster.
				if !is_ceremony_master(who) {
					debug!("TrustedGetter ceremonies_assignments, return: No master");
					return None
				}
				let assignments = EncointerCeremonies::assignments((community_id, ceremony_index));
				Some(assignments.encode())
			},
			TrustedGetter::ceremonies_meetup_participant_count_vote(
				who,
				community_id,
				ceremony_index,
				participant_account_id,
			) => {
				debug!("TrustedGetter ceremonies_meetup_participant_count_vote");
				// Todo Master or Me?
				// Block getter of confidential data if it is not the CeremonyMaster
				if !is_ceremony_master(who) {
					return None
				}
				Some(
					EncointerCeremonies::meetup_participant_count_vote(
						(community_id, ceremony_index),
						&participant_account_id,
					)
					.encode(),
				)
			},
			TrustedGetter::ceremonies_participant_attestees(
				who,
				community_id,
				ceremony_index,
				attestation_index,
			) => {
				debug!("TrustedGetter ceremonies_participant_attestees");
				// Block getter of confidential data if it is not the CeremonyMaster
				if !is_ceremony_master(who) {
					return None
				}
				match EncointerCeremonies::attestation_registry(
					(community_id, ceremony_index),
					&attestation_index,
				) {
					Some(b) => Some(b.encode()),
					_ => {
						warn!(
							"no attestees for {}, {}, {}",
							community_id, ceremony_index, attestation_index
						);
						None
					},
				}
			},
			TrustedGetter::ceremonies_participant_attestation_index(
				who,
				community_id,
				ceremony_index,
				participant_account_id,
			) => {
				debug!("TrustedGetter ceremonies_participant_attestation_index");
				//Todo Master or Me?
				// Block getter of confidential data if it is not the CeremonyMaster
				if !is_ceremony_master(who) {
					return None
				}
				Some(
					EncointerCeremonies::attestation_index(
						(community_id, ceremony_index),
						&participant_account_id,
					)
					.encode(),
				)
			},
			TrustedGetter::ceremonies_registered_bootstrapper(
				who,
				community_id,
				ceremony_index,
				participant_index,
			) => {
				debug!("TrustedGetter ceremonies_registered_bootstrapper");
				// Block getter of confidential data if it is not the CeremonyMaster.
				if !is_ceremony_master(who) {
					return None
				}
				match EncointerCeremonies::bootstrapper_registry(
					(community_id, ceremony_index),
					&participant_index,
				) {
					Some(b) => Some(b.encode()),
					_ => {
						warn!(
							"no bootstrapper for {}, {}, {}",
							community_id, ceremony_index, participant_index
						);
						None
					},
				}
			},
			TrustedGetter::ceremonies_registered_bootstrappers(
				who,
				community_id,
				ceremony_index,
			) => {
				debug!("TrustedGetter ceremonies_registered_bootstrappers");
				// Block getter of confidential data if it is not the CeremonyMaster.
				if !is_ceremony_master(who) {
					return None
				}
				let mut participants: Vec<AccountId> = Vec::new();

				let num_registered_bootstrappers =
					EncointerCeremonies::bootstrapper_count((community_id, ceremony_index));
				debug!("found {} bootstrappers ", num_registered_bootstrappers);
				if num_registered_bootstrappers < 1 {
					return None
				}

				for i in 0..num_registered_bootstrappers {
					match EncointerCeremonies::bootstrapper_registry(
						(community_id, ceremony_index),
						i + 1,
					) {
						Some(b) => {
							participants.push(b.clone());
						},
						_ => {
							warn!(
								"no bootstrapper for {}, {}, {}",
								community_id,
								ceremony_index,
								i + 1
							)
						},
					};
				}
				Some(participants.encode())
			},
			TrustedGetter::ceremonies_registered_reputable(
				who,
				community_id,
				ceremony_index,
				participant_index_type,
			) => {
				debug!("TrustedGetter ceremonies_registered_reputable");
				// Block getter of confidential data if it is not the CeremonyMaster.
				if !is_ceremony_master(who) {
					return None
				}
				match EncointerCeremonies::reputable_registry(
					(community_id, ceremony_index),
					&participant_index_type,
				) {
					Some(r) => Some(r.encode()),
					_ => {
						warn!(
							"no reputable for {}, {}, {}",
							community_id, ceremony_index, participant_index_type
						);
						None
					},
				}
			},
			TrustedGetter::ceremonies_registered_reputables(who, community_id, ceremony_index) => {
				debug!("TrustedGetter ceremonies_registered_reputables");
				// Block getter of confidential data if it is not the CeremonyMaster.
				if !is_ceremony_master(who) {
					return None
				}

				let mut participants: Vec<AccountId> = Vec::new();

				let num_registered_reputables =
					EncointerCeremonies::reputable_count((community_id, ceremony_index));
				debug!("found {} reputables ", num_registered_reputables);
				if num_registered_reputables < 1 {
					return None
				}

				for i in 0..num_registered_reputables {
					match EncointerCeremonies::reputable_registry(
						(community_id, ceremony_index),
						i + 1,
					) {
						Some(b) => {
							participants.push(b.clone());
						},
						_ => {
							warn!(
								"no reputable for {}, {}, {}",
								community_id,
								ceremony_index,
								i + 1
							);
						},
					};
				}
				Some(participants.encode())
			},
			TrustedGetter::ceremonies_registered_endorsee(
				who,
				community_id,
				ceremony_index,
				participant_index_type,
			) => {
				debug!("TrustedGetter ceremonies_registered_endorsee");
				// Block getter of confidential data if it is not the CeremonyMaster.
				if !is_ceremony_master(who) {
					return None
				}
				match EncointerCeremonies::endorsee_registry(
					(community_id, ceremony_index),
					&participant_index_type,
				) {
					Some(e) => Some(e.encode()),
					_ => {
						warn!(
							"no endorsee for {}, {}, {}",
							community_id, ceremony_index, participant_index_type
						);
						None
					},
				}
			},
			TrustedGetter::ceremonies_registered_endorsees(who, community_id, ceremony_index) => {
				debug!("TrustedGetter ceremonies_registered_endorsees");
				// Block getter of confidential data if it is not the CeremonyMaster.
				if !is_ceremony_master(who) {
					return None
				}

				let mut participants: Vec<AccountId> = Vec::new();

				let num_registered_endorsees =
					EncointerCeremonies::endorsee_count((community_id, ceremony_index));
				debug!("found {} endorsees ", num_registered_endorsees);
				if num_registered_endorsees < 1 {
					return None
				}

				for i in 0..num_registered_endorsees {
					match EncointerCeremonies::endorsee_registry(
						(community_id, ceremony_index),
						i + 1,
					) {
						Some(b) => {
							participants.push(b.clone());
						},
						_ => {
							warn!(
								"no endorsee for {}, {}, {}",
								community_id,
								ceremony_index,
								i + 1
							);
						},
					};
				}
				Some(participants.encode())
			},
			TrustedGetter::ceremonies_registered_newbie(
				who,
				community_id,
				ceremony_index,
				participant_index_type,
			) => {
				debug!("TrustedGetter ceremonies_registered_newbie");
				// Block getter of confidential data if it is not the CeremonyMaster.
				if !is_ceremony_master(who) {
					return None
				}
				match EncointerCeremonies::newbie_registry(
					(community_id, ceremony_index),
					&participant_index_type,
				) {
					Some(e) => Some(e.encode()),
					_ => {
						warn!(
							"no newbie for {}, {}, {}",
							community_id, ceremony_index, participant_index_type
						);
						None
					},
				}
			},
			TrustedGetter::ceremonies_registered_newbies(who, community_id, ceremony_index) => {
				debug!("TrustedGetter ceremonies_registered_newbies");
				// Block getter of confidential data if it is not the CeremonyMaster.
				if !is_ceremony_master(who) {
					return None
				}

				let mut participants: Vec<AccountId> = Vec::new();

				let num_registered_newbies =
					EncointerCeremonies::newbie_count((community_id, ceremony_index));
				debug!("found {} newbies ", num_registered_newbies);
				if num_registered_newbies < 1 {
					return None
				}

				for i in 0..num_registered_newbies {
					match EncointerCeremonies::newbie_registry(
						(community_id, ceremony_index),
						i + 1,
					) {
						Some(b) => {
							participants.push(b.clone());
						},
						_ => {
							warn!("no newbie for {}, {}, {}", community_id, ceremony_index, i + 1);
						},
					};
				}
				Some(participants.encode())
			},
			#[cfg(feature = "evm")]
			TrustedGetter::evm_nonce(who) => {
				let evm_account = get_evm_account(&who);
				let evm_account = HashedAddressMapping::into_account_id(evm_account);
				let nonce = System::account_nonce(&evm_account);
				debug!("TrustedGetter evm_nonce");
				debug!("Account nonce is {}", nonce);
				Some(nonce.encode())
			},
			#[cfg(feature = "evm")]
			TrustedGetter::evm_account_codes(_who, evm_account) =>
			// TODO: This probably needs some security check if who == evm_account (or assosciated)
				if let Some(info) = get_evm_account_codes(&evm_account) {
					debug!("TrustedGetter Evm Account Codes");
					debug!("AccountCodes for {} is {:?}", evm_account, info);
					Some(info) // TOOD: encoded?
				} else {
					None
				},
			#[cfg(feature = "evm")]
			TrustedGetter::evm_account_storages(_who, evm_account, index) =>
			// TODO: This probably needs some security check if who == evm_account (or assosciated)
				if let Some(value) = get_evm_account_storages(&evm_account, &index) {
					debug!("TrustedGetter Evm Account Storages");
					debug!("AccountStorages for {} is {:?}", evm_account, value);
					Some(value.encode())
				} else {
					None
				},
		}
	}

	fn get_storage_hashes_to_update(self) -> Vec<Vec<u8>> {
		debug!("get_storage_hashes_to_update for TrustedGetter");
		let mut key_hashes = Vec::new();
		match self.getter {
			TrustedGetter::encointer_balance(_, cid) => {
				key_hashes.push(storage_map_key(
					"EncointerBalances",
					"DemurragePerBlock",
					&cid,
					&StorageHasher::Blake2_128Concat,
				));
			},
			TrustedGetter::ceremonies_aggregated_account_data(_, _, _) => {
				key_hashes.push(storage_value_key("EncointerScheduler", "CurrentCeremonyIndex"));
				let current_phase =
					pallet_encointer_scheduler::Pallet::<ita_sgx_runtime::Runtime>::current_phase(); // updated im block import
				key_hashes.push(storage_map_key(
					"EncointerScheduler",
					"PhaseDurations",
					&current_phase,
					&StorageHasher::Blake2_128Concat,
				));
				key_hashes.push(storage_value_key("EncointerScheduler", " NextPhaseTimestamp"));
			},
			_ => {
				debug!("No storage updates needed...");
			},
		};
		key_hashes
	}
}

impl ExecuteGetter for PublicGetter {
	fn execute(self) -> Option<Vec<u8>> {
		match self {
			PublicGetter::some_value => Some(42u32.encode()),
			PublicGetter::encointer_total_issuance(community_id) => {
				let updated_total_issuance = pallet_encointer_balances::Pallet::<
					ita_sgx_runtime::Runtime,
				>::total_issuance(community_id);
				debug!("PublicGetter encointer_total_issuance");
				Some(updated_total_issuance.encode())
			},
			PublicGetter::ceremonies_assignment_counts(community_id, ceremony_index) => {
				let count = EncointerCeremonies::assignment_counts((community_id, ceremony_index));
				Some(count.encode())
			},
			PublicGetter::ceremonies_attestation_count(community_id, ceremony_index) => {
				let count = pallet_encointer_ceremonies::Pallet::<
					ita_sgx_runtime::Runtime,
				>::attestation_count((community_id, ceremony_index));
				Some(count.encode())
			},
			PublicGetter::ceremonies_meetup_count(community_id, ceremony_index) => {
				let count = EncointerCeremonies::meetup_count((community_id, ceremony_index));
				Some(count.encode())
			},
			PublicGetter::ceremonies_meetup_time_offset() => {
				let offset = EncointerCeremonies::meetup_time_offset();
				Some(offset.encode())
			},
			PublicGetter::ceremonies_registered_bootstrappers_count(
				community_id,
				ceremony_index,
			) => {
				let count = EncointerCeremonies::bootstrapper_count((community_id, ceremony_index));
				Some(count.encode())
			},
			PublicGetter::ceremonies_registered_reputables_count(community_id, ceremony_index) => {
				let count = EncointerCeremonies::reputable_count((community_id, ceremony_index));
				Some(count.encode())
			},
			PublicGetter::ceremonies_registered_newbies_count(community_id, ceremony_index) => {
				let count = EncointerCeremonies::newbie_count((community_id, ceremony_index));
				Some(count.encode())
			},
			PublicGetter::ceremonies_registered_endorsees_count(community_id, ceremony_index) => {
				let count = EncointerCeremonies::endorsees_count((community_id, ceremony_index));
				Some(count.encode())
			},
			PublicGetter::ceremonies_reward(community_id) => {
				let reward = EncointerCeremonies::nominal_income(&community_id);
				Some(reward.encode())
			},
		}
	}

	fn get_storage_hashes_to_update(self) -> Vec<Vec<u8>> {
		debug!("get_storage_hashes_to_update for PublicGetter");
		let mut key_hashes = Vec::new();
		match self {
			PublicGetter::ceremonies_reward(community_id) => {
				key_hashes.push(storage_map_key(
					"EncointerCommunities",
					"NominalIncome",
					&community_id,
					&StorageHasher::Blake2_128Concat,
				));
			},
			_ => {
				debug!("No storage updates needed...");
			},
		};
		key_hashes
	}
}
