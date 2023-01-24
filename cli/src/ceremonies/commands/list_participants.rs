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
	ceremonies::commands::ceremonies_command_utils::list_participants,
	command_utils::get_chain_api, trusted_command_utils::get_pair_from_str,
	trusted_commands::TrustedArgs, trusted_operation::perform_trusted_operation, Cli,
};
use encointer_primitives::communities::CommunityIdentifier;
use ita_stf::{KeyPair, TrustedGetter, TrustedOperation};
use itp_node_api::api_client::encointer::EncointerApi;
use log::*;
use sp_core::Pair;
use std::str::FromStr;

/// List registered participants for next encointer ceremony and supplied community identifier.  
#[derive(Debug, Clone, Parser)]
pub struct ListParticipantsCommand {
	/// Only Ceremony Master can execute this (SUDO).
	who: String,

	/// Community Id.
	community_id: String,
}

impl ListParticipantsCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let api = get_chain_api(cli);

		let who = get_pair_from_str(trusted_args, &self.who);

		info!("list participant ceremony for community {}", self.community_id);
		let community_identifier = CommunityIdentifier::from_str(&self.community_id).unwrap();
		let ceremony_index = api.get_current_ceremony_index(None).unwrap().unwrap();
		let top: TrustedOperation = TrustedGetter::ceremonies_registered_bootstrappers(
			who.public().into(),
			community_identifier,
			ceremony_index,
		)
		.sign(&KeyPair::Sr25519(who.clone()))
		.into();
		let bootstrappers = perform_trusted_operation(cli, trusted_args, &top);

		let top: TrustedOperation = TrustedGetter::ceremonies_registered_reputables(
			who.public().into(),
			community_identifier,
			ceremony_index,
		)
		.sign(&KeyPair::Sr25519(who.clone()))
		.into();
		let reputables = perform_trusted_operation(cli, trusted_args, &top);

		let top: TrustedOperation = TrustedGetter::ceremonies_registered_endorsees(
			who.public().into(),
			community_identifier,
			ceremony_index,
		)
		.sign(&KeyPair::Sr25519(who.clone()))
		.into();
		let endorsees = perform_trusted_operation(cli, trusted_args, &top);

		let top: TrustedOperation = TrustedGetter::ceremonies_registered_newbies(
			who.public().into(),
			community_identifier,
			ceremony_index,
		)
		.sign(&KeyPair::Sr25519(who))
		.into();
		let newbies = perform_trusted_operation(cli, trusted_args, &top);

		println!(
			"Participants of community {} for ceremony {} :",
			ceremony_index, self.community_id
		);
		println!("- Bootstrappers :");
		list_participants(bootstrappers);
		println!();
		println!("- Reputables :");
		list_participants(reputables);
		println!();
		println!("- Endorsees :");
		list_participants(endorsees);
		println!();
		println!("- Newbies :");
		list_participants(newbies);
	}
}
