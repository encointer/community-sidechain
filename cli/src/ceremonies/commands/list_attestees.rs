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

use crate::{
	ceremonies::commands::ceremonies_command_utils::{
		get_meetup_index, participant_attestation_index_map, AttestationState,
	},
	command_utils::get_chain_api,
	trusted_command_utils::get_pair_from_str,
	trusted_commands::TrustedArgs,
	trusted_operation::perform_trusted_operation,
	Cli,
};
use codec::Decode;
use encointer_primitives::{ceremonies::AttestationIndexType, communities::CommunityIdentifier};
use ita_stf::{KeyPair, PublicGetter, TrustedGetter, TrustedOperation};
use itp_node_api::api_client::encointer::EncointerApi;
use log::*;
use sp_core::Pair;
use std::{collections::HashMap, str::FromStr};

/// Listing all attestees for participants for supplied community identifier and ceremony index.
#[derive(Debug, Clone, Parser)]
pub struct ListAttesteesCommand {
	/// Participant : sender's on-chain AccountId in ss58check format.
	who: String,

	/// Community Id.
	community_id: String,
}

impl ListAttesteesCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let api = get_chain_api(cli);
		let who = get_pair_from_str(trusted_args, &self.who);

		let community_identifier = CommunityIdentifier::from_str(&self.community_id).unwrap();
		let ceremony_index = api.get_current_ceremony_index(None).unwrap().unwrap();

		println!(
			"Listing all attestees for community {} and ceremony {}",
			self.community_id, ceremony_index
		);

		let top: TrustedOperation =
			PublicGetter::ceremonies_attestation_count(community_identifier, ceremony_index).into();
		let encoded_attestee = perform_trusted_operation(cli, trusted_args, &top).unwrap();
		let attestee_count =
			AttestationIndexType::decode(&mut encoded_attestee.as_slice()).unwrap_or_default();
		println!("number of attestees:  {}", attestee_count);

		println!(
			"Get attestation index for all participants of cid {} and ceremony nr {}",
			community_identifier, ceremony_index
		);

		let mut participants_attestation_indexes = HashMap::new();

		let top: TrustedOperation = TrustedGetter::ceremonies_registered_bootstrappers(
			who.public().into(),
			community_identifier,
			ceremony_index,
		)
		.sign(&KeyPair::Sr25519(who.clone()))
		.into();
		match perform_trusted_operation(cli, trusted_args, &top) {
			Some(encoded_bootstrappers) => {
				participants_attestation_indexes.extend(participant_attestation_index_map(
					cli,
					trusted_args,
					&self.who,
					community_identifier,
					ceremony_index,
					encoded_bootstrappers,
				));
			},
			None => println!("No bootstrappers registered for this ceremony"),
		}

		let top: TrustedOperation = TrustedGetter::ceremonies_registered_reputables(
			who.public().into(),
			community_identifier,
			ceremony_index,
		)
		.sign(&KeyPair::Sr25519(who.clone()))
		.into();
		match perform_trusted_operation(cli, trusted_args, &top) {
			Some(encoded_reputables) => {
				participants_attestation_indexes.extend(participant_attestation_index_map(
					cli,
					trusted_args,
					&self.who,
					community_identifier,
					ceremony_index,
					encoded_reputables,
				));
			},
			None => println!("No reputables registered for this ceremony"),
		}

		let top: TrustedOperation = TrustedGetter::ceremonies_registered_endorsees(
			who.public().into(),
			community_identifier,
			ceremony_index,
		)
		.sign(&KeyPair::Sr25519(who.clone()))
		.into();
		match perform_trusted_operation(cli, trusted_args, &top) {
			Some(encoded_endorsees) => {
				participants_attestation_indexes.extend(participant_attestation_index_map(
					cli,
					trusted_args,
					&self.who,
					community_identifier,
					ceremony_index,
					encoded_endorsees,
				));
			},
			None => println!("No endorsees registered for this ceremony"),
		}

		let top: TrustedOperation = TrustedGetter::ceremonies_registered_newbies(
			who.public().into(),
			community_identifier,
			ceremony_index,
		)
		.sign(&KeyPair::Sr25519(who.clone()))
		.into();
		match perform_trusted_operation(cli, trusted_args, &top) {
			Some(encoded_newbies) => {
				participants_attestation_indexes.extend(participant_attestation_index_map(
					cli,
					trusted_args,
					&self.who,
					community_identifier,
					ceremony_index,
					encoded_newbies,
				));
			},
			None => println!("No newbies registered for this ceremony"),
		}

		let mut attestation_states = Vec::with_capacity(attestee_count as usize);

		for attestation_index in 1..attestee_count + 1 {
			let attestor = participants_attestation_indexes[&attestation_index].clone();
			info!("Create Attestation state for {:?}", attestor);
			let meetup_index = get_meetup_index(
				cli,
				trusted_args,
				&self.who,
				community_identifier,
				attestor.clone(),
			)
			.unwrap()
			.unwrap();

			let top: TrustedOperation = TrustedGetter::ceremonies_participant_attestees(
				who.public().into(),
				community_identifier,
				ceremony_index,
				attestation_index,
			)
			.sign(&KeyPair::Sr25519(who.clone()))
			.into();
			let encoded_index = perform_trusted_operation(cli, trusted_args, &top).unwrap();
			let attestees = Decode::decode(&mut encoded_index.as_slice()).unwrap();

			let top: TrustedOperation = TrustedGetter::ceremonies_meetup_participant_count_vote(
				who.public().into(),
				community_identifier,
				ceremony_index,
				attestor.clone(),
			)
			.sign(&KeyPair::Sr25519(who.clone()))
			.into();
			let encoded_count_vote =
				perform_trusted_operation(cli, trusted_args, &top).unwrap_or_default();
			let vote = Decode::decode(&mut encoded_count_vote.as_slice()).unwrap_or_default();

			let attestation_state = AttestationState::new(
				(community_identifier, ceremony_index),
				meetup_index,
				vote,
				attestation_index,
				attestor,
				attestees,
			);

			attestation_states.push(attestation_state);
		}

		// Group attestation states by meetup index
		attestation_states.sort_by(|a, b| a.meetup_index.partial_cmp(&b.meetup_index).unwrap());
		println!("Attestees :");
		for a in attestation_states.iter() {
			println!("{:?}", a);
		}
	}
}
