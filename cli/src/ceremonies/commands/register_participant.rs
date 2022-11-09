//todo: add license

use crate::{
	ceremonies::commands::ceremonies_command_utils::prove_attendance,
	command_utils::get_chain_api,
	get_layer_two_nonce,
	trusted_command_utils::{get_accountid_from_str, get_identifiers, get_pair_from_str},
	trusted_commands::TrustedArgs,
	trusted_operation::perform_trusted_operation,
	Cli,
};
use codec::{Decode, Encode};
use encointer_primitives::{
	ceremonies::ProofOfAttendance, communities::CommunityIdentifier, scheduler::CeremonyIndexType,
};
use ita_stf::{AccountId, Index, KeyPair, Signature, TrustedCall, TrustedGetter, TrustedOperation};
use itp_node_api::api_client::{encointer::EncointerApi, ParentchainApi};
use log::*;
use sp_application_crypto::sr25519;
use sp_core::{crypto::Ss58Codec, sr25519 as sr25519_core, Pair};

///Register participant for next encointer ceremony
#[derive(Debug, Clone, Parser)]
pub struct RegisterParticipantCommand {
	/// Participant : sender's on-chain AccountId in ss58check format.
	who: String,

	/// Prove attendance reputation for last ceremony
	reputation: Option<String>,
}

impl RegisterParticipantCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let api = get_chain_api(cli);

		let who = get_pair_from_str(trusted_args, &self.who);
		let accountid = get_accountid_from_str(&self.who);

		let (mrenclave, shard) = get_identifiers(trusted_args);
		let cid = api.get_community_identifier(shard, None).unwrap().unwrap();

		let proof = match &self.reputation {
			Some(r) => {
				let ceremony_index = api.get_current_ceremony_index(None).unwrap().unwrap();

				Some(prove_attendance(&accountid, cid, ceremony_index - 1, &who))
			},
			None => None,
		};

		info!("reputation: {:?}", proof);
		let nonce = get_layer_two_nonce!(who, cli, trusted_args);
		let top = TrustedCall::ceremonies_register_participant(who.public().into(), cid, proof)
			.sign(&KeyPair::Sr25519(who), nonce, &mrenclave, &shard)
			.into_trusted_operation(trusted_args.direct);

		let _ = perform_trusted_operation(cli, trusted_args, &top);
		info!("trusted call ceremonies_register_participant");
	}
}
