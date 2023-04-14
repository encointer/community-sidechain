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
	encointer_balances::commands::{
		set_fee_conversion_factor::SetFeeConversionFactorCommand, transfer_all::TransferAllCommand,
	},
	trusted_commands::TrustedArgs,
	Cli,
};

pub(crate) mod commands;

#[derive(Debug, clap::Subcommand)]
pub enum EncointerBalancesCommands {
	SetFeeConversionFactor(SetFeeConversionFactorCommand),
	CommunityTransferAll(TransferAllCommand),
}

impl EncointerBalancesCommands {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		match self {
			EncointerBalancesCommands::SetFeeConversionFactor(cmd) => cmd.run(cli, trusted_args),
			EncointerBalancesCommands::CommunityTransferAll(cmd) => cmd.run(cli, trusted_args),
		}
	}
}
