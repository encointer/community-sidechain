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
use codec::{Decode, Encode, Error as CodecError};
use encointer_ceremonies_assignment::assignment_fn_inverse;
use encointer_primitives::{
	ceremonies::{
		AggregatedAccountData, Assignment, AssignmentCount, AttestationIndexType,
		CommunityCeremony, MeetupIndexType, ParticipantIndexType, ParticipantType,
		ProofOfAttendance,
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
use std::collections::HashMap;

pub const ONE_DAY: Moment = 86_400_000;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{0}")]
	Codec(#[from] CodecError),
	#[error("Error, other: {0}")]
	Other(Box<dyn std::error::Error + Sync + Send + 'static>),
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct AttestationState {
	pub community_ceremony: CommunityCeremony,
	pub meetup_index: MeetupIndexType,
	pub vote: u32,
	pub attestation_index: u64,
	pub attestor: AccountId,
	pub attestees: Vec<AccountId>,
}

impl AttestationState {
	pub fn new(
		community_ceremony: CommunityCeremony,
		meetup_index: MeetupIndexType,
		vote: u32,
		attestation_index: u64,
		attestor: AccountId,
		attestees: Vec<AccountId>,
	) -> Self {
		Self { community_ceremony, meetup_index, vote, attestation_index, attestor, attestees }
	}
}

pub fn get_ceremony_stats(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
) -> Result<CommunityCeremonyStats, Error> {
	let api = get_chain_api(cli);
	let who = get_pair_from_str(trusted_args, arg_who);

	let top: TrustedOperation = TrustedGetter::ceremonies_assignments(
		who.public().into(),
		community_identifier,
		ceremony_index,
	)
	.sign(&KeyPair::Sr25519(who))
	.into();
	let encoded_assignments = perform_trusted_operation(cli, trusted_args, &top)
		.ok_or_else(|| Error::Other("Assignments don't exist".into()))?;
	let assignment = Decode::decode(&mut encoded_assignments.as_slice())?;

	let top: TrustedOperation =
		PublicGetter::ceremonies_meetup_count(community_identifier, ceremony_index).into();
	let encoded_meetup_count = perform_trusted_operation(cli, trusted_args, &top)
		.ok_or_else(|| Error::Other("MeetupCount not found".into()))?;
	let meetup_count = Decode::decode(&mut encoded_meetup_count.as_slice())?;

	let top: TrustedOperation = PublicGetter::ceremonies_meetup_time_offset().into();
	let encoded_meetup_time_offset =
		perform_trusted_operation(cli, trusted_args, &top).unwrap_or_default();
	let meetup_time_offset =
		Decode::decode(&mut encoded_meetup_time_offset.as_slice()).unwrap_or_default();

	let top: TrustedOperation =
		PublicGetter::ceremonies_assignment_counts(community_identifier, ceremony_index).into();
	let encoded_assigned = perform_trusted_operation(cli, trusted_args, &top)
		.ok_or_else(|| Error::Other("AssignmentCounts not found".into()))?;
	let assigned = Decode::decode(&mut encoded_assigned.as_slice())?;

	let mut meetups = vec![];
	for meetup_index in 1..=meetup_count {
		let meetup_location = api
			.get_meetup_locations(community_identifier, assignment, meetup_index)
			.expect("No meetup location found.")
			.unwrap();
		let time = api.get_meetup_time(meetup_location, ONE_DAY, meetup_time_offset).unwrap_or(0);

		//meetup participants
		let participants = get_meetup_participants_with_type(
			cli,
			trusted_args,
			arg_who,
			community_identifier,
			ceremony_index,
			meetup_index,
			meetup_count,
			assignment,
			assigned,
		)?;
		meetups.push(Meetup::new(meetup_index, meetup_location, time, participants))
	}

	Ok(CommunityCeremonyStats::new(
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
	account_id: AccountId,
) -> Result<AggregatedAccountData<AccountId, Moment>, Error> {
	let who = get_pair_from_str(trusted_args, arg_who);

	let top: TrustedOperation = TrustedGetter::ceremonies_aggregated_account_data(
		who.public().into(),
		community_identifier,
		account_id,
	)
	.sign(&KeyPair::Sr25519(who))
	.into();

	let encoded_data = perform_trusted_operation(cli, trusted_args, &top)
		.ok_or_else(|| Error::Other("AggregatedAccountData doesn't exist".into()))?;
	Ok(Decode::decode(&mut encoded_data.as_slice())?)
}

pub fn get_meetup_index(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	account_id: AccountId,
) -> Result<Option<MeetupIndexType>, Error> {
	let account_data =
		get_aggregated_account_data(cli, trusted_args, arg_who, community_identifier, account_id)?;
	match account_data.personal {
		Some(personal) => Ok(personal.meetup_index),
		None => Err(Error::Other("No personal data in AggregatedAccountData".into())),
	}
}

#[allow(clippy::too_many_arguments)]
fn get_meetup_participants_with_type(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	meetup_index: MeetupIndexType,
	meetup_count: MeetupIndexType,
	assignment: Assignment,
	assigned: AssignmentCount,
) -> Result<Vec<(AccountId, ParticipantType)>, Error> {
	let meetup_index_zero_based = meetup_index - 1;
	if meetup_index_zero_based > meetup_count {
		return Err(Error::Other(
			format!(
				"Invalid meetup index > meetup count: {}, {}",
				meetup_index_zero_based, meetup_count
			)
			.into(),
		))
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
		get_bootstrapper_or_reputable_with_type(
			cli,
			trusted_args,
			arg_who,
			community_identifier,
			ceremony_index,
			p_index,
			&assigned,
		)
		.ok()
	});

	let endorsees = assignment_fn_inverse(
		meetup_index_zero_based,
		assignment.endorsees,
		meetup_count,
		assigned.endorsees,
	)
	.unwrap_or_default()
	.into_iter()
	.filter(|p| p < &assigned.endorsees)
	.filter_map(|p| {
		get_endorsee_with_type(
			cli,
			trusted_args,
			arg_who,
			community_identifier,
			ceremony_index,
			p + 1,
		)
		.ok()
	});

	let newbies = assignment_fn_inverse(
		meetup_index_zero_based,
		assignment.newbies,
		meetup_count,
		assigned.newbies,
	)
	.unwrap_or_default()
	.into_iter()
	.filter(|p| p < &assigned.newbies)
	.filter_map(|p| {
		get_newbie_with_type(
			cli,
			trusted_args,
			arg_who,
			community_identifier,
			ceremony_index,
			p + 1,
		)
		.ok()
	});

	Ok(bootstrappers_reputables.chain(endorsees).chain(newbies).collect())
}

fn get_bootstrapper_or_reputable_with_type(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	participant_index: ParticipantIndexType,
	assigned: &AssignmentCount,
) -> Result<(AccountId, ParticipantType), Error> {
	if participant_index < assigned.bootstrappers {
		return Ok((
			get_bootstrapper(
				cli,
				trusted_args,
				arg_who,
				community_identifier,
				ceremony_index,
				participant_index + 1,
			)?,
			ParticipantType::Bootstrapper,
		))
	} else if participant_index < assigned.bootstrappers + assigned.reputables {
		return Ok((
			get_reputable(
				cli,
				trusted_args,
				arg_who,
				community_identifier,
				ceremony_index,
				participant_index - assigned.bootstrappers + 1,
			)?,
			ParticipantType::Reputable,
		))
	}
	Err(Error::Other(
		format!("Bootstrapper or Reputable at index {} not found", participant_index).into(),
	))
}

fn get_bootstrapper(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	participant_index: ParticipantIndexType,
) -> Result<AccountId, Error> {
	let who = get_pair_from_str(trusted_args, arg_who);
	let top: TrustedOperation = TrustedGetter::ceremonies_registered_bootstrapper(
		who.public().into(),
		community_identifier,
		ceremony_index,
		participant_index,
	)
	.sign(&KeyPair::Sr25519(who))
	.into();
	let encoded_bootstrapper =
		perform_trusted_operation(cli, trusted_args, &top).ok_or_else(|| {
			Error::Other(format!("Bootstrapper at index {} not found", participant_index).into())
		})?;

	Ok(Decode::decode(&mut encoded_bootstrapper.as_slice())?)
}

fn get_reputable(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	participant_index: ParticipantIndexType,
) -> Result<AccountId, Error> {
	let who = get_pair_from_str(trusted_args, arg_who);
	let top: TrustedOperation = TrustedGetter::ceremonies_registered_reputable(
		who.public().into(),
		community_identifier,
		ceremony_index,
		participant_index,
	)
	.sign(&KeyPair::Sr25519(who))
	.into();
	let encoded_reputable =
		perform_trusted_operation(cli, trusted_args, &top).ok_or_else(|| {
			Error::Other(format!("Reputable at index {} not found", participant_index).into())
		})?;

	Ok(Decode::decode(&mut encoded_reputable.as_slice())?)
}

fn get_endorsee_with_type(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	participant_index: ParticipantIndexType,
) -> Result<(AccountId, ParticipantType), Error> {
	let who = get_pair_from_str(trusted_args, arg_who);
	let top: TrustedOperation = TrustedGetter::ceremonies_registered_endorsee(
		who.public().into(),
		community_identifier,
		ceremony_index,
		participant_index,
	)
	.sign(&KeyPair::Sr25519(who))
	.into();
	let encoded_endorsee = perform_trusted_operation(cli, trusted_args, &top).ok_or_else(|| {
		Error::Other(format!("Endorsee at index {} not found", participant_index).into())
	})?;

	Ok((Decode::decode(&mut encoded_endorsee.as_slice())?, ParticipantType::Endorsee))
}

fn get_newbie_with_type(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	participant_index: ParticipantIndexType,
) -> Result<(AccountId, ParticipantType), Error> {
	let who = get_pair_from_str(trusted_args, arg_who);
	let top: TrustedOperation = TrustedGetter::ceremonies_registered_newbie(
		who.public().into(),
		community_identifier,
		ceremony_index,
		participant_index,
	)
	.sign(&KeyPair::Sr25519(who))
	.into();
	let encoded_newbie = perform_trusted_operation(cli, trusted_args, &top).ok_or_else(|| {
		Error::Other(format!("Newbie at index {} not found", participant_index).into())
	})?;

	Ok((Decode::decode(&mut encoded_newbie.as_slice())?, ParticipantType::Newbie))
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

pub fn participant_attestation_index_map(
	cli: &Cli,
	trusted_args: &TrustedArgs,
	arg_who: &str,
	community_identifier: CommunityIdentifier,
	ceremony_index: CeremonyIndexType,
	encoded_participants: Vec<u8>,
) -> HashMap<AttestationIndexType, AccountId> {
	let mut participants_attestation_indexes = HashMap::new();
	let who = get_pair_from_str(trusted_args, arg_who);
	match decode_participants(Some(encoded_participants)) {
		Some(p) =>
			for account_id in p {
				let top: TrustedOperation =
					TrustedGetter::ceremonies_participant_attestation_index(
						who.public().into(),
						community_identifier,
						ceremony_index,
						account_id.clone(),
					)
					.sign(&KeyPair::Sr25519(who.clone()))
					.into();
				if let Some(encoded_index) = perform_trusted_operation(cli, trusted_args, &top) {
					if let Ok(index) = AttestationIndexType::decode(&mut encoded_index.as_slice()) {
						participants_attestation_indexes.insert(index, account_id);
					}
				}
			},
		None => {
			error!("participant_attestation_index_map: Couldn't decode participants");
		},
	};
	participants_attestation_indexes
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
