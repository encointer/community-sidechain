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
	command_utils::get_chain_api, trusted_command_utils::get_pair_from_str,
	trusted_commands::TrustedArgs, trusted_operation::perform_trusted_operation, Cli,
};
use codec::{Decode, Encode};
use encointer_ceremonies_assignment::assignment_fn_inverse;
use encointer_primitives::{
	ceremonies::{
		AggregatedAccountData, Assignment, AssignmentCount, CommunityCeremony, MeetupIndexType,
		MeetupTimeOffsetType, ParticipantIndexType, ParticipantType, ProofOfAttendance,
	},
	communities::{CommunityIdentifier, Location},
	scheduler::CeremonyIndexType,
};
use ita_stf::{
	AccountId, KeyPair, Moment, PublicGetter, Signature, TrustedGetter, TrustedOperation,
};
use itp_node_api::api_client::encointer::EncointerApi;
use log::*;
use serde::{Deserialize, Serialize};
use sp_application_crypto::Ss58Codec;
use sp_core::{sr25519 as sr25519_core, Pair};

pub const ONE_DAY: Moment = 86_400_000;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meetup {
	pub index: MeetupIndexType,
	pub location: Location,
	pub time: Moment,
	pub registrations: Vec<(AccountId, ParticipantType)>,
}

impl Meetup {
	pub fn new(
		index: MeetupIndexType,
		location: Location,
		time: Moment,
		registrations: Vec<(AccountId, ParticipantType)>,
	) -> Self {
		Self { index, location, time, registrations }
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityCeremonyStats {
	pub community_ceremony: CommunityCeremony,
	pub assignment: Assignment,
	pub assignment_count: AssignmentCount,
	pub meetup_count: MeetupIndexType,
	pub meetups: Vec<Meetup>,
}

impl CommunityCeremonyStats {
	pub fn new(
		community_ceremony: CommunityCeremony,
		assignment: Assignment,
		assignment_count: AssignmentCount,
		meetup_count: MeetupIndexType,
		meetups: Vec<Meetup>,
	) -> Self {
		Self { community_ceremony, assignment, assignment_count, meetup_count, meetups }
	}
}

pub fn get_ceremony_stats(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
) -> Option<CommunityCeremonyStats> {
	error!("get_ceremony_stats in");
	let api = get_chain_api(cli);
	let who = get_pair_from_str(trusted_args, arg_who);

	let top: TrustedOperation = TrustedGetter::ceremonies_assignments(
		who.public().into(),
		community_identifier,
		ceremony_index,
	)
	.sign(&KeyPair::Sr25519(who.clone()))
	.into();
	let encoded_assignments = perform_trusted_operation(cli, trusted_args, &top);
	let assignment = decode_assignments(encoded_assignments).unwrap_or_default();
	error!("found assignment: bootstarppers_reputables.m{}", assignment.bootstrappers_reputables.m);

	let top: TrustedOperation =
		PublicGetter::ceremonies_meetup_count(community_identifier, ceremony_index).into();

	let mut meetup_count = MeetupIndexType::default();
	if let Some(mcount) = perform_trusted_operation(cli, trusted_args, &top) {
		match MeetupIndexType::decode(&mut mcount.as_slice()) {
			Ok(mc) => {
				meetup_count = mc;
				error!("found meetup_count: {}", meetup_count);
			},
			Err(e) => {
				error!("Could not decode the meetup count");
			},
		}
	} else {
		println!("meetup count: unknown");
	};

	let top: TrustedOperation = PublicGetter::ceremonies_meetup_time_offset().into();
	let mut meetup_time_offset = MeetupTimeOffsetType::default();
	if let Some(offset) = perform_trusted_operation(cli, trusted_args, &top) {
		match MeetupTimeOffsetType::decode(&mut offset.as_slice()) {
			Ok(o) => {
				meetup_time_offset = o;
			},
			Err(e) => {
				error!("Could not decode the meetup time offset");
			},
		}
	} else {
		println!("meetup time offset: unknown");
	};

	let top: TrustedOperation =
		PublicGetter::ceremonies_assignment_counts(community_identifier, ceremony_index).into();
	let mut assigned = AssignmentCount::default();
	if let Some(count) = perform_trusted_operation(cli, trusted_args, &top) {
		match AssignmentCount::decode(&mut count.as_slice()) {
			Ok(ac) => {
				assigned = ac;
				error!("found assignment_count: {}", assigned.get_number_of_participants());
			},
			Err(e) => {
				error!("Could not decode the assignment count");
			},
		}
	} else {
		println!("assignment count: unknown");
	};

	let mut meetups = vec![];
	for meetup_index in 1..=meetup_count {
		error!("Check meetup {}", meetup_index);
		let meetup_location = api
			.get_meetup_locations(community_identifier, assignment, meetup_index)
			.unwrap_or_default()
			.unwrap_or_default();
		let time = api.get_meetup_time(meetup_location, ONE_DAY, meetup_time_offset).unwrap_or(0);

		error!("get meetup_participants: ");
		//meetup participants
		let participants = get_meetup_participants(
			cli,
			trusted_args,
			arg_who,
			community_identifier,
			ceremony_index,
			meetup_index,
			meetup_count,
			assignment,
			assigned,
		);
		error!("found {} meetup_participants: ", participants.len());
		let mut registrations = vec![];

		for participant in participants.into_iter() {
			let registration = get_registration(
				cli,
				trusted_args,
				arg_who,
				community_identifier,
				ceremony_index,
				participant.clone(),
			)?;
			registrations.push((participant, registration))
		}
		error!("push meetup to meetups:");
		meetups.push(Meetup::new(meetup_index, meetup_location, time, registrations))
	}

	error!("generate stats with {} meetups", meetups.len());
	Some(CommunityCeremonyStats::new(
		(community_identifier, ceremony_index),
		assignment,
		assigned,
		meetup_count,
		meetups,
	))
}

fn get_aggregated_account_data(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	account_id: AccountId,
) -> Option<AggregatedAccountData<AccountId, Moment>> {
	//signer: Master or Me (account_id)
	let who = get_pair_from_str(trusted_args, arg_who);

	//TODO account sign and param?
	let top: TrustedOperation = TrustedGetter::ceremonies_aggregated_account_data(
		account_id,
		community_identifier,
		ceremony_index,
	)
	.sign(&KeyPair::Sr25519(who.clone()))
	.into();

	let encoded_data = perform_trusted_operation(cli, trusted_args, &top);
	decode_aggregated_account_data(encoded_data)
}

fn get_meetup_index(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	account_id: AccountId,
) -> Option<MeetupIndexType> {
	if let Some(account_data) = get_aggregated_account_data(
		cli,
		trusted_args,
		arg_who,
		community_identifier,
		ceremony_index,
		account_id,
	) {
		match account_data.personal {
			Some(personal) => return personal.meetup_index,
			None => return None,
		}
	} else {
		println!("aggregated account data: unknown");
	}
	None
}

fn get_registration(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	account_id: AccountId,
) -> Option<ParticipantType> {
	if let Some(account_data) = get_aggregated_account_data(
		cli,
		trusted_args,
		arg_who,
		community_identifier,
		ceremony_index,
		account_id.clone(),
	) {
		match account_data.personal {
			Some(personal) => return Some(personal.participant_type),
			None => {
				println!(
					"Participant {:?} is not registered for ceremony {} of community {}",
					&account_id, ceremony_index, community_identifier
				);
				return None
			},
		};
	} else {
		error!("Could not get aggregated account data for {:?}", &account_id);
	}
	None
}

fn get_meetup_participants(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	meetup_index: MeetupIndexType,
	meetup_count: MeetupIndexType,
	assignment: Assignment,
	assigned: AssignmentCount,
) -> Vec<AccountId> {
	let meetup_index_zero_based = meetup_index - 1;
	if meetup_index_zero_based > meetup_count {
		error!(
			"Invalid meetup index > meetup count: {}, {}",
			meetup_index_zero_based, meetup_count
		);
		return vec![]
	};
	let bootstrappers_reputables = assignment_fn_inverse(
		meetup_index_zero_based,
		assignment.bootstrappers_reputables,
		meetup_count,
		assigned.bootstrappers + assigned.reputables,
	)
	.unwrap_or_default()
	.into_iter()
	.filter_map(|p_index| {
		get_bootstrapper_or_reputable(
			cli,
			trusted_args,
			arg_who,
			community_identifier,
			ceremony_index,
			p_index,
			&assigned,
		)
	});
	bootstrappers_reputables.collect()
}

fn get_bootstrapper_or_reputable(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	p_index: ParticipantIndexType,
	assigned: &AssignmentCount,
) -> Option<AccountId> {
	if p_index < assigned.bootstrappers {
		return get_bootstrapper(
			cli,
			trusted_args,
			arg_who,
			community_identifier,
			ceremony_index,
			p_index + 1,
		)
	} else if p_index < assigned.bootstrappers + assigned.reputables {
		return get_reputable(
			cli,
			trusted_args,
			arg_who,
			community_identifier,
			ceremony_index,
			p_index - assigned.bootstrappers + 1,
		)
	}
	None
}

fn get_bootstrapper(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	participant_index_type: ParticipantIndexType,
) -> Option<AccountId> {
	let who = get_pair_from_str(trusted_args, arg_who);
	let top: TrustedOperation = TrustedGetter::ceremonies_registered_bootstrapper(
		who.public().into(),
		community_identifier,
		ceremony_index,
		participant_index_type,
	)
	.sign(&KeyPair::Sr25519(who.clone()))
	.into();
	let bootstrapper = perform_trusted_operation(cli, trusted_args, &top);
	decode_participant(bootstrapper)
}

fn get_reputable(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	participant_index_type: ParticipantIndexType,
) -> Option<AccountId> {
	let who = get_pair_from_str(trusted_args, arg_who);
	let top: TrustedOperation = TrustedGetter::ceremonies_registered_reputable(
		who.public().into(),
		community_identifier,
		ceremony_index,
		participant_index_type,
	)
	.sign(&KeyPair::Sr25519(who.clone()))
	.into();
	let reputable = perform_trusted_operation(cli, trusted_args, &top);
	decode_participant(reputable)
}

pub fn prove_attendance(
	prover: &AccountId,
	cid: CommunityIdentifier,
	cindex: CeremonyIndexType,
	attendee: &sr25519_core::Pair,
) -> ProofOfAttendance<Signature, AccountId> {
	let msg = (prover.clone(), cindex);
	debug!("generating proof of attendance for {} and cindex: {}", prover, cindex);
	debug!("signature payload is {:x?}", msg.encode());
	ProofOfAttendance {
		prover_public: prover.clone(),
		ceremony_index: cindex,
		community_identifier: cid,
		attendee_public: AccountId::from(attendee.public()),
		attendee_signature: Signature::from(attendee.sign(&msg.encode())),
	}
}

pub fn list_participants(encoded_participants: Option<Vec<u8>>) {
	match decode_participants(encoded_participants) {
		Some(p) =>
			for account in p {
				println!("    {}", account.to_ss58check());
			},
		None => {
			println!("    No one");
		},
	};
}
pub fn decode_aggregated_account_data(
	encoded_data: Option<Vec<u8>>,
) -> Option<AggregatedAccountData<AccountId, Moment>> {
	encoded_data.and_then(|data| {
		if let Ok(data_decoded) = Decode::decode(&mut data.as_slice()) {
			Some(data_decoded)
		} else {
			error!("Could not decode the aggregated account data");
			None
		}
	})
}

pub fn decode_participant(encoded_participant: Option<Vec<u8>>) -> Option<AccountId> {
	encoded_participant.and_then(|p| {
		if let Ok(account_decoded) = Decode::decode(&mut p.as_slice()) {
			Some(account_decoded)
		} else {
			error!("Could not decode the participants");
			None
		}
	})
}

pub fn decode_participants(encoded_participants: Option<Vec<u8>>) -> Option<Vec<AccountId>> {
	encoded_participants.and_then(|participants| {
		if let Ok(account_decoded) = Decode::decode(&mut participants.as_slice()) {
			Some(account_decoded)
		} else {
			error!("Could not decode the participants");
			None
		}
	})
}

pub fn decode_assignments(encoded_assignments: Option<Vec<u8>>) -> Option<Assignment> {
	encoded_assignments.and_then(|assignments| {
		if let Ok(assignment_decoded) = Decode::decode(&mut assignments.as_slice()) {
			Some(assignment_decoded)
		} else {
			error!("Could not decode the assignments");
			None
		}
	})
}
