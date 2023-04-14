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
	trusted_command_utils::{get_identifiers, get_pair_from_str},
	trusted_commands::TrustedArgs,
	trusted_operation::perform_trusted_operation,
	Cli,
};
use codec::Decode;
use encointer_primitives::balances::FeeConversionFactorType;
use ita_stf::{Index, KeyPair, TrustedCall, TrustedGetter, TrustedOperation};
use log::*;
use sp_core::{crypto::Ss58Codec, Pair};

/// Set fee conversion factor
#[derive(Debug, Clone, Parser)]
pub struct SetFeeConversionFactorCommand {
	/// sender's AccountId in ss58check format
	who: String,

	/// fee conversion factor
	factor: FeeConversionFactorType,
}

impl SetFeeConversionFactorCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let who = get_pair_from_str(trusted_args, &self.who);
		println!("from ss58 is public {}", who.public().to_ss58check());

		println!(
			"send trusted set fee conversion factor to {}. Sent by {} ",
			self.factor,
			who.public(),
		);
		let (mrenclave, shard) = get_identifiers(trusted_args);
		let nonce = get_layer_two_nonce!(who, cli, trusted_args);
		let top =
			TrustedCall::encointer_set_fee_conversion_factor(who.public().into(), self.factor)
				.sign(&KeyPair::Sr25519(who), nonce, &mrenclave, &shard)
				.into_trusted_operation(trusted_args.direct);

		let _ = perform_trusted_operation(cli, trusted_args, &top);
		println!("trusted call encointer_set_fee_conversion_factor executed");
	}
}
