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
	ceremonies::commands::{
		attest_attendees::AttestAttendeesCommand, claim_rewards::ClaimRewardsCommand,
		community_infos::CommunityInfosCommand, list_attestees::ListAttesteesCommand,
		list_meetups::ListMeetupsCommand, list_participants::ListParticipantsCommand,
		register_participant::RegisterParticipantCommand,
		upgrade_registration::UpgradeRegistrationCommand,
	},
	trusted_commands::TrustedArgs,
	Cli,
};

mod commands;

#[derive(Debug, clap::Subcommand)]
pub enum CeremoniesCommands {
	AttestAttendees(AttestAttendeesCommand),
	ClaimRewards(ClaimRewardsCommand),
	CommunityInfos(CommunityInfosCommand),
	ListAttestees(ListAttesteesCommand),
	ListMeetups(ListMeetupsCommand),
	ListParticipants(ListParticipantsCommand),
	RegisterParticipant(RegisterParticipantCommand),
	UpgradeRegistration(UpgradeRegistrationCommand),
	/*
		CeremoniesUnregisterParticipant(),
		CeremoniesAttestAttendees(),
		CeremoniesAttestClaims(),
		CeremoniesEndorseNewcomer(),
		CeremoniesSetInactivityTimeout(),
		CeremoniesSetEndorsementTicketsPerBootstrapper(),
		CeremoniesSetEndorsementTicketsPerReputable(),
		CeremoniesSetReputationLifetime(),
		CeremoniesSetMeetupTimeOffset(),
		CeremoniesSetTimeTolerance(),
		CeremoniesSetLocationTolerance(),
		CeremoniesPurgeCommunityCeremony(),
	*/
}

impl CeremoniesCommands {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		match self {
			CeremoniesCommands::AttestAttendees(cmd) => cmd.run(cli, trusted_args),
			CeremoniesCommands::ClaimRewards(cmd) => cmd.run(cli, trusted_args),
			CeremoniesCommands::CommunityInfos(cmd) => cmd.run(cli, trusted_args),
			CeremoniesCommands::ListAttestees(cmd) => cmd.run(cli, trusted_args),
			CeremoniesCommands::ListMeetups(cmd) => cmd.run(cli, trusted_args),
			CeremoniesCommands::ListParticipants(cmd) => cmd.run(cli, trusted_args),
			CeremoniesCommands::RegisterParticipant(cmd) => cmd.run(cli, trusted_args),
			CeremoniesCommands::UpgradeRegistration(cmd) => cmd.run(cli, trusted_args),
		}
	}
}
