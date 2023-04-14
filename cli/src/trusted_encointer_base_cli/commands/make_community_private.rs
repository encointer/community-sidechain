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
use encointer_primitives::communities::CommunityIdentifier;
use ita_stf::{Index, KeyPair, TrustedCall, TrustedGetter, TrustedOperation};
use itp_node_api::api_client::encointer::EncointerApi;
use log::*;
use sp_core::{crypto::Ss58Codec, Pair};
use std::str::FromStr;

/// Make community private. Can only be called in registering phase by the ceremony master.
#[derive(Debug, Clone, Parser)]
pub struct MakeCommunityPrivateCommand {
	/// Ceremony Master : sender's on-chain AccountId in ss58check format.
	who: String,

	/// Community Id.
	community_id: String,
}

impl MakeCommunityPrivateCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let api = get_chain_api(cli);

		let who = get_pair_from_str(trusted_args, &self.who);
		info!("who ss58 is {}", who.public().to_ss58check());

		let (mrenclave, shard) = get_identifiers(trusted_args);

		info!("community_id {}", self.community_id);

		let cid = CommunityIdentifier::from_str(&self.community_id).unwrap();

		//Update Locations
		let locations = api.get_community_locations(cid).unwrap();
		info!("{} locations to migrate: ", locations.len());

		let nonce = get_layer_two_nonce!(who, cli, trusted_args);

		info!(
			"who {} send trusted call ceremonies_migrate_to_private_community {}",
			who.public(),
			cid,
		);
		let top = TrustedCall::ceremonies_migrate_to_private_community(
			who.public().into(),
			cid,
			locations,
		)
		.sign(&KeyPair::Sr25519(who), nonce, &mrenclave, &shard)
		.into_trusted_operation(trusted_args.direct);
		let _ = perform_trusted_operation(cli, trusted_args, &top).unwrap();
		info!("trusted call ceremonies_migrate_to_private_community executed");
	}
}
