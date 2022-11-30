//todo: add license

use crate::{
	ceremonies::commands::{
		register_participant::RegisterParticipantCommand,
		upgrade_registration::UpgradeRegistrationCommand,
	},
	trusted_commands::TrustedArgs,
	Cli,
};

mod commands;

#[derive(Debug, clap::Subcommand)]
pub enum CeremoniesCommands {
	RegisterParticipant(RegisterParticipantCommand),
	UpgradeRegistration(UpgradeRegistrationCommand),
	/*
		CeremoniesUnregisterParticipant(),
		CeremoniesAttestAttendees(),
		CeremoniesAttestClaims(),
		CeremoniesEndorseNewcomer(),
		CeremoniesClaimRewards(),
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
			CeremoniesCommands::RegisterParticipant(cmd) => cmd.run(cli, trusted_args),
			CeremoniesCommands::UpgradeRegistration(cmd) => cmd.run(cli, trusted_args),
		}
	}
}
