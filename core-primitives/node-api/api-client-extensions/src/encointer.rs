//todo: add license

use crate::ApiResult;
use encointer_primitives::{
	communities::{CommunityIdentifier, GeoHash},
	scheduler::CeremonyIndexType,
};
use itp_types::{AccountId, ShardIdentifier};
use sp_core::{Pair, H256 as Hash};
use sp_runtime::MultiSignature;
use substrate_api_client::{Api, ExtrinsicParams, RpcClient};

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
}
