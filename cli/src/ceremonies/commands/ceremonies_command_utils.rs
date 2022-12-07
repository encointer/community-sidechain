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

use codec::{Decode, Encode};
use encointer_primitives::{
	ceremonies::ProofOfAttendance, communities::CommunityIdentifier, scheduler::CeremonyIndexType,
};
use ita_stf::{AccountId, Signature};
use log::*;
use sp_application_crypto::Ss58Codec;
use sp_core::{sr25519 as sr25519_core, Pair};

pub fn prove_attendance(
	prover: &AccountId,
	cid: CommunityIdentifier,
	cindex: CeremonyIndexType,
	attendee: &sr25519_core::Pair,
) -> ProofOfAttendance<Signature, AccountId> {
	let msg = (prover.clone(), cindex);
	debug!("generating proof of attendance for {} and cindex: {}", prover, cindex);
	debug!("signature payload is {:x?}", msg.encode());
	ProofOfAttendance {
		prover_public: prover.clone(),
		ceremony_index: cindex,
		community_identifier: cid,
		attendee_public: AccountId::from(attendee.public()),
		attendee_signature: Signature::from(attendee.sign(&msg.encode())),
	}
}

pub fn list_participants(encoded_participants: Option<Vec<u8>>) {
	match decode_participants(encoded_participants) {
		Some(p) =>
			for account in p {
				println!("    {}", account.to_ss58check());
			},
		None => {
			println!("    No one");
		},
	};
}

pub fn decode_participants(encoded_participants: Option<Vec<u8>>) -> Option<Vec<AccountId>> {
	encoded_participants.and_then(|participants| {
		if let Ok(account_decoded) = Decode::decode(&mut participants.as_slice()) {
			Some(account_decoded)
		} else {
			error!("Could not decode the participants");
			None
		}
	})
}
