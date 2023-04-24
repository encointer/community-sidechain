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
	encointer_balances::commands::balances_command_utils::decode_encointer_balance,
	trusted_command_utils::{get_balance, get_pair_from_str},
	trusted_commands::TrustedArgs,
	trusted_operation::perform_trusted_operation,
	Cli,
};
use encointer_primitives::communities::{CommunityIdentifier, LossyInto};
use ita_stf::{KeyPair, TrustedGetter, TrustedOperation};
use sp_core::Pair;
use std::str::FromStr;

#[derive(Parser)]
pub struct BalanceCommand {
	/// AccountId in ss58check format
	account: String,

	/// Optional Community Id. If it is supplied, returns balance in that community. Otherwise balance of parentchain native token"
	community_id: Option<String>,
}

impl BalanceCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		match &self.community_id {
			Some(cid) => {
				let who = get_pair_from_str(trusted_args, &self.account);
				let cid = CommunityIdentifier::from_str(cid).unwrap();
				let top: TrustedOperation =
					TrustedGetter::encointer_balance(who.public().into(), cid)
						.sign(&KeyPair::Sr25519(who))
						.into();
				let res = perform_trusted_operation(cli, trusted_args, &top);
				let balance_type = decode_encointer_balance(res).unwrap_or_default();
				let amount: f64 = balance_type.lossy_into();
				println!("{}", amount);
			},
			None => {
				println!("{}", get_balance(cli, trusted_args, &self.account).unwrap_or_default());
			},
		}
	}
}
