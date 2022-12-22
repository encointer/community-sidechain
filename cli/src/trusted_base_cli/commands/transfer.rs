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

use crate::{
	get_layer_two_nonce,
	trusted_command_utils::{get_accountid_from_str, get_identifiers, get_pair_from_str},
	trusted_commands::TrustedArgs,
	trusted_operation::perform_trusted_operation,
	Cli,
};
use codec::Decode;
use encointer_primitives::{balances::BalanceType, communities::CommunityIdentifier};
use ita_stf::{Index, KeyPair, TrustedCall, TrustedGetter, TrustedOperation};
use log::*;
use my_node_runtime::Balance;
use sp_core::{crypto::Ss58Codec, Pair};
use std::str::FromStr;

#[derive(Parser)]
pub struct TransferCommand {
	/// sender's AccountId in ss58check format
	from: String,

	/// recipient's AccountId in ss58check format
	to: String,

	/// amount to be transferred
	amount: Balance,

	/// Optional Community Id. If it is supplied, transfers balance in that community currency. Otherwise send parentchain native token"
	community_id: Option<String>,
}

impl TransferCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let from = get_pair_from_str(trusted_args, &self.from);
		let to = get_accountid_from_str(&self.to);
		info!("from ss58 is {}", from.public().to_ss58check());
		info!("to ss58 is {}", to.to_ss58check());

		println!("send trusted call transfer from {} to {}: {}", from.public(), to, self.amount);
		let (mrenclave, shard) = get_identifiers(trusted_args);
		let nonce = get_layer_two_nonce!(from, cli, trusted_args);

		let top = match &self.community_id {
			Some(cid) => {
				let cid = CommunityIdentifier::from_str(cid).unwrap();
				println!("in community {} currency", cid);
				TrustedCall::encointer_balance_transfer(
					from.public().into(),
					to,
					cid,
					BalanceType::from_num(self.amount),
				)
				.sign(&KeyPair::Sr25519(from), nonce, &mrenclave, &shard)
				.into_trusted_operation(trusted_args.direct)
			},
			None => TrustedCall::balance_transfer(from.public().into(), to, self.amount)
				.sign(&KeyPair::Sr25519(from), nonce, &mrenclave, &shard)
				.into_trusted_operation(trusted_args.direct),
		};
		let _ = perform_trusted_operation(cli, trusted_args, &top);
		info!("trusted call transfer executed");
	}
}
