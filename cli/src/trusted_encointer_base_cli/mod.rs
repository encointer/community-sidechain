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
	trusted_commands::TrustedArgs,
	trusted_encointer_base_cli::commands::make_community_private::MakeCommunityPrivateCommand, Cli,
};

mod commands;

#[derive(Debug, clap::Subcommand)]
pub enum TrustedEncointerBaseCli {
	MakeCommunityPrivate(MakeCommunityPrivateCommand),
}

impl TrustedEncointerBaseCli {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		match self {
			TrustedEncointerBaseCli::MakeCommunityPrivate(cmd) => cmd.run(cli, trusted_args),
		}
	}
}
