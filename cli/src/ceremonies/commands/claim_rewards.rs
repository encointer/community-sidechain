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
	command_utils::get_chain_api,
	get_layer_two_nonce,
	trusted_command_utils::{get_identifiers, get_pair_from_str},
	trusted_commands::TrustedArgs,
	trusted_operation::perform_trusted_operation,
	Cli,
};
use codec::Decode;
use encointer_primitives::{
	ceremonies::MeetupIndexType, communities::CommunityIdentifier, scheduler::CeremonyPhaseType,
};
use ita_stf::{Index, KeyPair, PublicGetter, TrustedCall, TrustedGetter, TrustedOperation};
use itp_node_api::api_client::encointer::EncointerApi;
use log::*;
use sp_core::Pair;
use std::str::FromStr;

/// Claim the rewards for all participants of last ceremony's specified meetup(s).
#[derive(Debug, Clone, Parser)]
pub struct ClaimRewardsCommand {
	/// Participant : sender's on-chain AccountId in ss58check format.
	who: String,

	/// Community Id.
	community_id: String,

	/// Meetup index, if "all" isn't set.
	/// If None, claim the rewards for all participants in the same meetup as the extrinsic signer "who".
	meetup_index: Option<String>,

	/// Insert if claim the rewards for all meetups of last ceremony.  
	all: bool,
}
// todo : check payment, see encointer node
impl ClaimRewardsCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let api = get_chain_api(cli);

		let who = get_pair_from_str(trusted_args, &self.who);
		let (mrenclave, shard) = get_identifiers(trusted_args);

		let community_identifier = CommunityIdentifier::from_str(&self.community_id).unwrap();

		let mut nonce = get_layer_two_nonce!(who, cli, trusted_args);

		if self.all {
			info!("Claim the rewards for all participants of all meetups {}", self.community_id);
			let mut ceremony_index = api.get_current_ceremony_index(None).unwrap().unwrap();
			if api.get_current_phase().unwrap() == CeremonyPhaseType::Registering {
				ceremony_index -= 1;
			}
			let top: TrustedOperation =
				PublicGetter::ceremonies_meetup_count(community_identifier, ceremony_index).into();
			let encoded_meetup_count = perform_trusted_operation(cli, trusted_args, &top).unwrap();
			let meetup_count = Decode::decode(&mut encoded_meetup_count.as_slice()).unwrap();
			for meetup_index in 1..=meetup_count {
				let top = TrustedCall::ceremonies_claim_rewards(
					who.public().into(),
					community_identifier,
					Some(meetup_index),
				)
				.sign(&KeyPair::Sr25519(who.clone()), nonce, &mrenclave, &shard)
				.into_trusted_operation(trusted_args.direct);

				let _ = perform_trusted_operation(cli, trusted_args, &top);
				nonce += 1;
			}
			info!("Claiming reward for all meetup indexes executed ");
		} else {
			info!("Claim the rewards for all participants of 1 meetup");
			let meetup_index = match &self.meetup_index {
				Some(i) => MeetupIndexType::from_str(i).unwrap().into(),
				None => None,
			};
			let top = TrustedCall::ceremonies_claim_rewards(
				who.public().into(),
				community_identifier,
				meetup_index,
			)
			.sign(&KeyPair::Sr25519(who), nonce, &mrenclave, &shard)
			.into_trusted_operation(trusted_args.direct);

			let _ = perform_trusted_operation(cli, trusted_args, &top);
			info!("Claiming reward for all participant of meetup {:?} executed ", meetup_index);
		}
	}
}
