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

use crate::ApiResult;
use encointer_primitives::{
	ceremonies::{Assignment, MeetupIndexType, MeetupTimeOffsetType},
	communities::{CommunityIdentifier, GeoHash, Location},
	scheduler::{CeremonyIndexType, CeremonyPhaseType},
};
use itp_types::{AccountId, Moment};
use serde_json::json;
use sp_core::{Pair, H256 as Hash};
use sp_runtime::MultiSignature;
use substrate_api_client::{Api, ApiClientError, ExtrinsicParams, RpcClient};

pub const ENCOINTER_SCHEDULER: &str = "EncointerScheduler";
pub const COMMUNITIES: &str = "EncointerCommunities";

/// ApiClient extension that contains some convenience specific methods for encointer.
pub trait EncointerApi {
	fn get_bootstrappers(
		&self,
		community_id: CommunityIdentifier,
		at_block: Option<Hash>,
	) -> Vec<AccountId>;
	fn get_community_identifiers(&self, at_block: Option<Hash>) -> Vec<CommunityIdentifier>;
	fn get_community_identifier(
		&self,
		geo_hash: GeoHash,
		at_block: Option<Hash>,
	) -> CommunityIdentifier;
	fn get_current_ceremony_index(
		&self,
		at_block: Option<Hash>,
	) -> ApiResult<Option<CeremonyIndexType>>;
	fn get_next_phase_timestamp(&self) -> ApiResult<Moment>;
	fn get_current_phase(&self) -> ApiResult<CeremonyPhaseType>;
	fn get_phase_duration(&self, phase: CeremonyPhaseType) -> ApiResult<Moment>;
	fn get_community_locations(
		&self,
		community_id: CommunityIdentifier,
	) -> ApiResult<Vec<Location>>;
	fn get_meetup_locations(
		&self,
		community_id: CommunityIdentifier,
		assignment: Assignment,
		meetup_index: MeetupIndexType,
	) -> ApiResult<Option<Location>>;

	fn get_start_of_attesting_phase(&self) -> ApiResult<Moment>;
	fn get_meetup_time(
		&self,
		location: Location,
		one_day: Moment,
		meetup_time_offset: MeetupTimeOffsetType,
	) -> ApiResult<Moment>;
}

//TODO: get key and pallet names from metadata?
impl<P: Pair, Client: RpcClient, Params: ExtrinsicParams> EncointerApi for Api<P, Client, Params>
where
	MultiSignature: From<P::Signature>,
{
	fn get_bootstrappers(
		&self,
		community_id: CommunityIdentifier,
		at_block: Option<Hash>,
	) -> Vec<AccountId> {
		let result: Vec<AccountId> = self
			.get_storage_map(COMMUNITIES, "Bootstrappers", community_id, at_block)
			.unwrap()
			.unwrap();
		result
	}

	fn get_community_identifiers(&self, at_block: Option<Hash>) -> Vec<CommunityIdentifier> {
		let cids: Vec<CommunityIdentifier> = self
			.get_storage_value(COMMUNITIES, "CommunityIdentifiers", at_block)
			.unwrap()
			.expect("no community registered");
		cids
	}

	fn get_community_identifier(
		&self,
		geo_hash: GeoHash,
		at_block: Option<Hash>,
	) -> CommunityIdentifier {
		let result: CommunityIdentifier = self
			.get_storage_map(COMMUNITIES, "CommunityIdentifiersByGeohash", geo_hash, at_block)
			.unwrap()
			.unwrap();
		result
	}

	fn get_current_ceremony_index(
		&self,
		at_block: Option<Hash>,
	) -> ApiResult<Option<CeremonyIndexType>> {
		self.get_storage_value(ENCOINTER_SCHEDULER, "CurrentCeremonyIndex", at_block)
	}

	fn get_next_phase_timestamp(&self) -> ApiResult<Moment> {
		self.get_storage_value(ENCOINTER_SCHEDULER, "NextPhaseTimestamp", None)?
			.ok_or_else(|| ApiClientError::Other("Couldn't get NextPhaseTimestamp".into()))
	}

	fn get_current_phase(&self) -> ApiResult<CeremonyPhaseType> {
		self.get_storage_value(ENCOINTER_SCHEDULER, "CurrentPhase", None)?
			.ok_or_else(|| ApiClientError::Other("Couldn't get CurrentPhase".into()))
	}

	fn get_phase_duration(&self, phase: CeremonyPhaseType) -> ApiResult<Moment> {
		self.get_storage_map("ENCOINTER_SCHEDULER", "PhaseDurations", phase, None)?
			.ok_or_else(|| ApiClientError::Other("Couldn't get PhaseDurations".into()))
	}

	fn get_community_locations(
		&self,
		community_id: CommunityIdentifier,
	) -> ApiResult<Vec<Location>> {
		let req = json!({
		"method": "encointer_getLocations",
		"params": vec![community_id],
		"jsonrpc": "2.0",
		"id": "1",
		});

		let locations = self.get_request(req.into())?.ok_or_else(|| {
			ApiClientError::Other(
				format!("No locations founds. Does the cid {} exist", community_id).into(),
			)
		})?;

		serde_json::from_str(&locations).map_err(|e| ApiClientError::Other(e.into()))
	}

	fn get_meetup_locations(
		&self,
		community_id: CommunityIdentifier,
		assignment: Assignment,
		meetup_index: MeetupIndexType,
	) -> ApiResult<Option<Location>> {
		let locations = self.get_community_locations(community_id)?;
		let location_assignment_params = assignment.locations;
		let location = encointer_ceremonies_assignment::meetup_location(
			meetup_index,
			locations,
			location_assignment_params,
		);
		Ok(location)
	}

	fn get_start_of_attesting_phase(&self) -> ApiResult<Moment> {
		let next_phase_timestamp = self.get_next_phase_timestamp()?;

		match self.get_current_phase()? {
			CeremonyPhaseType::Assigning => Ok(next_phase_timestamp), // - next_phase_timestamp.rem(ONE_DAY),
			CeremonyPhaseType::Attesting => {
				self.get_phase_duration(CeremonyPhaseType::Attesting)
					.map(|dur| next_phase_timestamp - dur) //- next_phase_timestamp.rem(ONE_DAY)
			},
			CeremonyPhaseType::Registering => Err(ApiClientError::Other(
				"ceremony phase must be Assigning or Attesting to request meetup location.".into(),
			)),
		}
	}

	fn get_meetup_time(
		&self,
		location: Location,
		one_day: Moment,
		meetup_time_offset: MeetupTimeOffsetType,
	) -> ApiResult<Moment> {
		let attesting_start = self.get_start_of_attesting_phase()?;
		Ok(encointer_ceremonies_assignment::meetup_time(
			location,
			attesting_start,
			one_day,
			meetup_time_offset,
		))
	}
}
