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
	ceremonies::commands::ceremonies_command_utils::get_ceremony_stats,
	command_utils::get_chain_api, trusted_commands::TrustedArgs, Cli,
};
use encointer_primitives::communities::CommunityIdentifier;
use itp_node_api::api_client::encointer::EncointerApi;
use log::*;
use std::str::FromStr;

/// List all assigned meetups for current encointer ceremony and supplied community identifier.  
#[derive(Debug, Clone, Parser)]
pub struct ListMeetupsCommand {
	/// Only Ceremony Master can execute this (SUDO).
	who: String,

	/// Community Id.
	community_id: String,
}

impl ListMeetupsCommand {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let api = get_chain_api(cli);

		let community_identifier = CommunityIdentifier::from_str(&self.community_id).unwrap();
		let ceremony_index = api.get_current_ceremony_index(None).unwrap().unwrap();
		info!(
			"Listing meetups for community {} and ceremony {:?}",
			self.community_id, ceremony_index
		);

		let stats =
			get_ceremony_stats(cli, trusted_args, &self.who, community_identifier, ceremony_index)
				.unwrap();
		let mut num_assignees = 0u64;
		if stats.meetups.is_empty() {
			println!("No meetup. Is the community private ?");
		}
		for meetup in stats.meetups.iter() {
			println!(
				"MeetupRegistry[({}, {}), {}] location is {:?}, {:?}",
				self.community_id,
				ceremony_index,
				meetup.index,
				meetup.location.lat,
				meetup.location.lon
			);

			println!(
				"MeetupRegistry[({}, {}), {}] meeting time is {:?}",
				self.community_id, ceremony_index, meetup.index, meetup.time
			);

			if !meetup.registrations.is_empty() {
				let num = meetup.registrations.len();
				num_assignees += num as u64;
				println!(
					"MeetupRegistry[({}, {}), {}] participants: {}",
					self.community_id, ceremony_index, meetup.index, num
				);
				for (participant, registration) in meetup.registrations.iter() {
					println!("   {} as {:?}", participant, registration);
				}
			} else {
				println!(
					"MeetupRegistry[({}, {}), {}] EMPTY",
					self.community_id, ceremony_index, meetup.index
				);
			}
		}
		println!("total number of assignees: {}", num_assignees);
	}
}
