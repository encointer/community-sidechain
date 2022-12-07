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
	ceremonies::commands::ceremonies_command_utils::prove_attendance,
	command_utils::get_chain_api,
	get_layer_two_nonce,
	trusted_command_utils::{get_accountid_from_str, get_identifiers, get_pair_from_str},
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

///Register participant for next encointer ceremony
#[derive(Debug, Clone, Parser)]
pub struct RegisterParticipantCommand {
	/// Participant : sender's on-chain AccountId in ss58check format.
	who: String,

	/// Community Id
	community_id: String,

	/// Prove attendance reputation for last ceremony
	reputation: Option<String>,
}

impl RegisterParticipantCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let api = get_chain_api(cli);

		let who = get_pair_from_str(trusted_args, &self.who);
		let accountid = get_accountid_from_str(&self.who);
		debug!("from ss58 is public {}", who.public().to_ss58check());

		let (mrenclave, shard) = get_identifiers(trusted_args);

		debug!("community_id {}", self.community_id);

		let cid = CommunityIdentifier::from_str(&self.community_id).unwrap();

		let proof = match &self.reputation {
			Some(_r) => {
				let ceremony_index = api.get_current_ceremony_index(None).unwrap().unwrap();

				Some(prove_attendance(&accountid, cid, ceremony_index - 1, &who))
			},
			None => None,
		};

		debug!("reputation: {:?}", proof);
		let nonce = get_layer_two_nonce!(who, cli, trusted_args);
		let top = TrustedCall::ceremonies_register_participant(who.public().into(), cid, None)
			.sign(&KeyPair::Sr25519(who), nonce, &mrenclave, &shard)
			.into_trusted_operation(trusted_args.direct);

		let _ = perform_trusted_operation(cli, trusted_args, &top);
		debug!("trusted call ceremonies_register_participant executed");
	}
}
