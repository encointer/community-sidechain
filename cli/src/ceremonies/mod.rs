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

#[allow(clippy::enum_variant_names)]
#[derive(Subcommand)]
pub enum CeremoniesCommands {
	CeremoniesRegisterParticipant(RegisterParticipantCommand),
	CeremoniesUpgradeRegistration(UpgradeRegistrationCommand),
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
			CeremoniesCommands::CeremoniesRegisterParticipant(cmd) => cmd.run(cli, trusted_args),
			CeremoniesCommands::CeremoniesUpgradeRegistration(cmd) => cmd.run(cli, trusted_args),
		}
	}
}
