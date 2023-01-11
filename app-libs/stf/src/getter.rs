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
use encointer_primitives::{communities::CommunityIdentifier, scheduler::CeremonyIndexType};
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

#[cfg(feature = "evm")]
use sp_core::{H160, H256};

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
	/*
	   encointer_total_issuance(CommunityIdentifier),
	   ceremonies_registered_bootstrappers_count(CommunityIdentifier),
	   ceremonies_registered_reputables_count(CommunityIdentifier),
	   ceremonies_registered_endorsees_count(CommunityIdentifier),
	   ceremonies_registered_newbies_count(CommunityIdentifier),
	   ceremonies_meetup_count(CommunityIdentifier),
	   ceremonies_reward(CommunityIdentifier),
	   ceremonies_location_tolerance(CommunityIdentifier),
	   ceremonies_time_tolerance(CommunityIdentifier),
	   encointer_scheduler_state(CommunityIdentifier),
	*/
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TrustedGetter {
	free_balance(AccountId),
	reserved_balance(AccountId),
	nonce(AccountId),
	//encointer_balance(AccountId, CommunityIdentifier),
	//ceremonies_participant_index(AccountId, CommunityIdentifier),
	//Not public : ceremonies_meetup_index(AccountId, CommunityIdentifier),
	//ceremonies_reputations(AccountId, CommunityIdentifier),
	//ceremonies_attestations(AccountId, CommunityIdentifier),
	//ceremonies_aggregated_account_data(AccountId, CommunityIdentifier),
	ceremonies_registered_bootstrappers(AccountId, CommunityIdentifier, CeremonyIndexType),
	ceremonies_registered_reputables(AccountId, CommunityIdentifier, CeremonyIndexType),
	ceremonies_registered_endorsees(AccountId, CommunityIdentifier, CeremonyIndexType),
	ceremonies_registered_newbies(AccountId, CommunityIdentifier, CeremonyIndexType),
	//from ceremonies_aggregated_account_data: ceremonies_meetup_time_and_location(AccountId, CommunityIdentifier),
	//from ceremonies_aggregated_account_data: ceremonies_registration(AccountId, CommunityIdentifier),
	//Ceremonie Master
	//ceremonies_meetups(AccountId, CommunityIdentifier),
	//ceremonies_attestees(AccountId, CommunityIdentifier),
	//todo meetup_registery? ceremonies_meetup_registry(AccountId, CommunityIdentifier),
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
			//TrustedGetter::encointer_balance(sender_account, _) => sender_account,
			//TrustedGetter::ceremonies_participant_index(sender_account, _) => sender_account,
			//TrustedGetter::ceremonies_attestations(sender_account, _) => sender_account,
			//TrustedGetter::ceremonies_aggregated_account_data(sender_account, _) => sender_account,
			TrustedGetter::ceremonies_registered_bootstrappers(sender_account, _, _) =>
				sender_account,
			TrustedGetter::ceremonies_registered_reputables(sender_account, _, _) => sender_account,
			TrustedGetter::ceremonies_registered_endorsees(sender_account, _, _) => sender_account,
			TrustedGetter::ceremonies_registered_newbies(sender_account, _, _) => sender_account,
			//TrustedGetter::ceremonies_meetups(sender_account, _) => sender_account,
			//TrustedGetter::ceremonies_attestees(sender_account, _) => sender_account,
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
			/*
			TrustedGetter::encointer_balance(who, currency_id) => {
				let balance: BalanceEntry<BlockNumber> = pallet_encointer_balances::Pallet::<
					ita_sgx_runtime::Runtime,
				>::balance_entry(currency_id, who);
				debug!("TrustedGetter encointer_balance");
				Some(balance.encode())
			},
			TrustedGetter::ceremonies_participant_index(who, currency_id) => {
				let ceremony_index = pallet_encointer_scheduler::Pallet::<
					ita_sgx_runtime::Runtime,
				>::current_ceremony_index();
				let part: ParticipantIndexType = pallet_encointer_ceremonies::Pallet::<
					ita_sgx_runtime::Runtime,
				>::participant_index(
					(currency_id, ceremony_index),
					AccountId::from(who),
				);
				Some(part.encode())
			},
			TrustedGetter::ceremonies_attestations(who, community_id) => {
				let ceremony_index = pallet_encointer_scheduler::Pallet::<
					ita_sgx_runtime::Runtime,
				>::current_ceremony_index();
				let attestation_index = pallet_encointer_ceremonies::Pallet::<
					ita_sgx_runtime::Runtime,
				>::attestation_index(
					(community_id, ceremony_index), who
				);
				let attestations = pallet_encointer_ceremonies::Pallet::<
					ita_sgx_runtime::Runtime,
				>::attestation_registry(
					(community_id, ceremony_index), attestation_index
				);
				Some(attestations.encode())
			},
			TrustedGetter::ceremonies_aggregated_account_data(who, community_id) => {
				let aggregated_account_data = pallet_encointer_ceremonies::Pallet::<
					ita_sgx_runtime::Runtime,
				>::get_aggregated_account_data(community_id, &who);
				aggregated_account_data.personal.map(|p| p.encode())
			},

			 */
			TrustedGetter::ceremonies_registered_bootstrappers(
				who,
				community_id,
				ceremony_index,
			) => {
				debug!("TrustedGetter ceremonies_registered_bootstrappers");
				//Block getter of confidential data if it is not the CeremonyMaster
				if !is_ceremony_master(who) {
					return None
				}
				let mut participants: Vec<AccountId> = Vec::new();

				let num_registered_bootstrappers = pallet_encointer_ceremonies::Pallet::<
					ita_sgx_runtime::Runtime,
				>::bootstrapper_count((
					community_id,
					ceremony_index,
				));
				debug!("found {} bootstrappers ", num_registered_bootstrappers);
				if num_registered_bootstrappers < 1 {
					return None
				}

				for i in 0..num_registered_bootstrappers {
					match pallet_encointer_ceremonies::Pallet::<
						ita_sgx_runtime::Runtime,
					>::bootstrapper_registry(
						(community_id, ceremony_index), i + 1,
					) {
						Some(b) => {
							participants.push(b.clone());
						}
						_ => { warn!("no bootstrapper for {}, {}, {}", community_id, ceremony_index, i+1) }
					};
				}
				Some(participants.encode())
			},
			TrustedGetter::ceremonies_registered_reputables(who, community_id, ceremony_index) => {
				debug!("TrustedGetter ceremonies_registered_reputables");
				//Block getter of confidential data if it is not the CeremonyMaster
				if !is_ceremony_master(who) {
					return None
				}

				let mut participants: Vec<AccountId> = Vec::new();

				let num_registered_reputables = pallet_encointer_ceremonies::Pallet::<
					ita_sgx_runtime::Runtime,
				>::reputable_count((community_id, ceremony_index));
				debug!("found {} reputables ", num_registered_reputables);
				if num_registered_reputables < 1 {
					return None
				}

				for i in 0..num_registered_reputables {
					match pallet_encointer_ceremonies::Pallet::<
						ita_sgx_runtime::Runtime,
					>::reputable_registry(
						(community_id, ceremony_index), i + 1,
					) {
						Some(b) => {
							participants.push(b.clone());
						}
						_ => { warn!("no reputable for {}, {}, {}", community_id, ceremony_index, i+1) }
					};
				}
				Some(participants.encode())
			},
			TrustedGetter::ceremonies_registered_endorsees(who, community_id, ceremony_index) => {
				debug!("TrustedGetter ceremonies_registered_endorsees");
				//Block getter of confidential data if it is not the CeremonyMaster
				if !is_ceremony_master(who) {
					return None
				}

				let mut participants: Vec<AccountId> = Vec::new();

				let num_registered_endorsees = pallet_encointer_ceremonies::Pallet::<
					ita_sgx_runtime::Runtime,
				>::endorsee_count((community_id, ceremony_index));
				debug!("found {} endorsees ", num_registered_endorsees);
				if num_registered_endorsees < 1 {
					return None
				}

				for i in 0..num_registered_endorsees {
					match pallet_encointer_ceremonies::Pallet::<
						ita_sgx_runtime::Runtime,
					>::endorsee_registry(
						(community_id, ceremony_index), i + 1,
					) {
						Some(b) => {
							participants.push(b.clone());
						}
						_ => { warn!("no endorsee for {}, {}, {}", community_id, ceremony_index, i+1) }
					};
				}
				Some(participants.encode())
			},
			TrustedGetter::ceremonies_registered_newbies(who, community_id, ceremony_index) => {
				debug!("TrustedGetter ceremonies_registered_newbies");
				//Block getter of confidential data if it is not the CeremonyMaster
				if !is_ceremony_master(who) {
					return None
				}

				let mut participants: Vec<AccountId> = Vec::new();

				let num_registered_newbies = pallet_encointer_ceremonies::Pallet::<
					ita_sgx_runtime::Runtime,
				>::newbie_count((community_id, ceremony_index));
				debug!("found {} newbies ", num_registered_newbies);
				if num_registered_newbies < 1 {
					return None
				}

				for i in 0..num_registered_newbies {
					match pallet_encointer_ceremonies::Pallet::<
						ita_sgx_runtime::Runtime,
					>::newbie_registry(
						(community_id, ceremony_index), i + 1,
					) {
						Some(b) => {
							participants.push(b.clone());
						}
						_ => { warn!("no newbie for {}, {}, {}", community_id, ceremony_index, i+1) }
					};
				}
				Some(participants.encode())
			},
			/*
			TrustedGetter::ceremonies_meetups(who, community_id) => {
				//Master check
			},
			TrustedGetter::ceremonies_attestees(who, community_id) => {
				//Master check
			},

			 */
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
		Vec::new()
	}
}

impl ExecuteGetter for PublicGetter {
	fn execute(self) -> Option<Vec<u8>> {
		match self {
			PublicGetter::some_value => Some(42u32.encode()),
		}
	}

	fn get_storage_hashes_to_update(self) -> Vec<Vec<u8>> {
		Vec::new()
	}
}
