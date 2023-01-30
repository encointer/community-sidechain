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
	get_layer_two_nonce,
	trusted_command_utils::{get_accountid_from_str, get_identifiers, get_pair_from_str},
	trusted_commands::TrustedArgs,
	trusted_operation::perform_trusted_operation,
	Cli,
};
use codec::Decode;
use encointer_primitives::communities::CommunityIdentifier;
use ita_stf::{Index, KeyPair, TrustedCall, TrustedGetter, TrustedOperation};
use log::*;
use sp_core::Pair;
use std::str::FromStr;

/// Attest Encointer ceremony attendees claim for the supplied community
#[derive(Debug, Clone, Parser)]
pub struct AttestAttendeesCommand {
	/// Participant : sender's on-chain AccountId in ss58check format.
	who: String,

	/// Community Id.
	community_id: String,

	/// Participants who attested: list of AccountIds in ss58check format.
	#[structopt(min_values = 2)]
	attestations: Vec<String>,
}

impl AttestAttendeesCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let who = get_pair_from_str(trusted_args, &self.who);
		let (mrenclave, shard) = get_identifiers(trusted_args);

		info!(
			"Attest attendees claim for community {} and participant {}",
			self.community_id,
			who.public()
		);

		let community_identifier = CommunityIdentifier::from_str(&self.community_id).unwrap();

		let mut attestations = Vec::new();

		for a in &self.attestations {
			attestations.push(get_accountid_from_str(a));
		}

		let number_of_participants_vote = attestations.len() as u32 + 1u32;
		info!("Number of  vote {}", number_of_participants_vote);

		let nonce = get_layer_two_nonce!(who, cli, trusted_args);
		let top = TrustedCall::ceremonies_attest_attendees(
			who.public().into(),
			community_identifier,
			number_of_participants_vote,
			attestations,
		)
		.sign(&KeyPair::Sr25519(who), nonce, &mrenclave, &shard)
		.into_trusted_operation(trusted_args.direct);

		let _ = perform_trusted_operation(cli, trusted_args, &top);
		debug!("trusted call ceremonies_attest_attendees executed");
	}
}
