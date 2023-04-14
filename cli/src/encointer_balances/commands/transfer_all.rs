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
use sp_core::{crypto::Ss58Codec, Pair};
use std::str::FromStr;

/// Transfer the entire balance to another account in the same community
#[derive(Debug, Clone, Parser)]
pub struct TransferAllCommand {
	/// sender's AccountId in ss58check format
	from: String,

	/// recipient's AccountId in ss58check format
	to: String,

	/// Community Id.
	community_id: String,
}

impl TransferAllCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let from = get_pair_from_str(trusted_args, &self.from);
		let to = get_accountid_from_str(&self.to);
		println!("from ss58 is public {}", from.public().to_ss58check());
		println!("to ss58 is {}", to.to_ss58check());
		let cid = CommunityIdentifier::from_str(&self.community_id).unwrap();

		println!(
			"send trusted call transfer all from {} to {} of community {}",
			from.public(),
			to,
			self.community_id,
		);
		let (mrenclave, shard) = get_identifiers(trusted_args);
		let nonce = get_layer_two_nonce!(from, cli, trusted_args);
		let top = TrustedCall::encointer_balance_transfer_all(from.public().into(), to, cid)
			.sign(&KeyPair::Sr25519(from), nonce, &mrenclave, &shard)
			.into_trusted_operation(trusted_args.direct);

		let _ = perform_trusted_operation(cli, trusted_args, &top);
		println!("trusted call encointer_balance_transfer_all executed");
	}
}
