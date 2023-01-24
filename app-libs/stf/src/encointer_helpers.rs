/*
	Copyright 2022 Encointer Association

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

use crate::{helpers::get_storage_value, AccountId};
use encointer_primitives::{communities::CommunityIdentifier, scheduler::CeremonyPhaseType};
use frame_support::traits::EnsureOrigin;
use itp_storage::storage_value_key;
use itp_utils::stringify::account_id_to_string;
use log::*;
use std::prelude::v1::*;

pub fn is_ceremony_master(account_id: AccountId) -> bool {
	let origin = ita_sgx_runtime::Origin::signed(account_id.clone());
	match <ita_sgx_runtime::Runtime as pallet_encointer_ceremonies::Config>::CeremonyMaster::ensure_origin(
        origin,
    ) {
        Ok(_) => true,
        Err(_e) => {
            error!("bad origin: Confidential data can only be requested by the ceremony master: {}",  account_id_to_string(&account_id));
            false
        }
    }
}

/// For now, a community is private when its community identifier is stored in the communities pallet on the sidechain.
pub fn is_private_community(community_identifier: &CommunityIdentifier) -> bool {
	match private_community_ids() {
		Some(cids) => cids.contains(community_identifier),
		None => false,
	}
}

pub fn private_community_ids_storage_key() -> Vec<u8> {
	storage_value_key("EncointerCommunities", "CommunityIdentifiers")
}

pub fn private_community_ids() -> Option<Vec<CommunityIdentifier>> {
	get_storage_value("EncointerCommunities", "CommunityIdentifiers")
}

pub fn current_ceremony_phase_storage_key() -> Vec<u8> {
	storage_value_key("EncointerScheduler", "CurrentPhase")
}

pub fn current_ceremony_phase() -> Option<CeremonyPhaseType> {
	get_storage_value("EncointerScheduler", "CurrentPhase")
}
