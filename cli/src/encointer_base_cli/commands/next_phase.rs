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
	command_utils::{get_chain_api, get_pair_from_str},
	Cli,
};

use log::*;
use sp_core::{crypto::Ss58Codec, sr25519 as sr25519_core, Pair};
use substrate_api_client::{compose_call, compose_extrinsic, UncheckedExtrinsicV4, XtStatus};

#[derive(Debug, Clone, Parser)]
pub struct NextPhaseCommand {
	/// sender's AccountId in ss58check format
	from: String,
}

impl NextPhaseCommand {
	pub(crate) fn run(&self, cli: &Cli) {
		let from_account = get_pair_from_str(&self.from);
		info!("from ss58 is {}", from_account.public().to_ss58check());

		let api = get_chain_api(cli).set_signer(sr25519_core::Pair::from(from_account.clone()));

		let next_phase_call = compose_call!(api.metadata, "EncointerScheduler", "next_phase");

		let xt: UncheckedExtrinsicV4<_, _> =
			compose_extrinsic!(api, "Sudo", "sudo", next_phase_call);
		// send and watch extrinsic until finalized
		info!("Master {} trigger manually next phase ", from_account.public().to_ss58check());
		let tx_hash = api.send_extrinsic(xt.hex_encode(), XtStatus::Finalized).unwrap();
		info!("[+] Next Phase got finalized. Hash: {:?}\n", tx_hash);
	}
}
