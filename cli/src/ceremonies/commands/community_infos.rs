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
	encointer_balances::commands::balances_command_utils::decode_encointer_balance,
	trusted_commands::TrustedArgs, trusted_operation::perform_trusted_operation, Cli,
};
use codec::Decode;
use encointer_primitives::{
	balances::EncointerBalanceConverter,
	communities::{CommunityIdentifier, NominalIncome},
};
use ita_stf::{PublicGetter, TrustedOperation};
use sp_runtime::traits::Convert;
use std::str::FromStr;

/// List various public information for an encointer community.  
#[derive(Debug, Clone, Parser)]
pub struct CommunityInfosCommand {
	/// sender's AccountId in ss58check format.
	who: String,

	/// Community Id.
	community_id: String,
}

impl CommunityInfosCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		println!("Public information about community {}", self.community_id);
		let community_identifier = CommunityIdentifier::from_str(&self.community_id).unwrap();

		let top: TrustedOperation =
			PublicGetter::encointer_total_issuance(community_identifier).into();
		let encoded_total_issuance = perform_trusted_operation(cli, trusted_args, &top);
		let total_issuance = decode_encointer_balance(encoded_total_issuance);
		println!(
			"Total inssuance {}",
			EncointerBalanceConverter::convert(total_issuance.unwrap_or_default()),
		);

		let top: TrustedOperation = PublicGetter::ceremonies_reward(community_identifier).into();
		let encoded_reward = perform_trusted_operation(cli, trusted_args, &top).unwrap();
		let reward = NominalIncome::decode(&mut encoded_reward.as_slice()).unwrap_or_default();
		println!("Reward {} ", reward);

		//Todo:
		//location tolerance
		//time tolerance
		//participant count
		//meetup count
		//scheduler state
	}
}
