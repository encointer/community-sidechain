//todo: add license

use crate::ApiResult;
use encointer_primitives::{
	communities::{CommunityIdentifier, GeoHash},
	scheduler::CeremonyIndexType,
};
use itp_types::ShardIdentifier;
use sp_core::{Pair, H256 as Hash};
use sp_runtime::MultiSignature;
use substrate_api_client::{Api, ExtrinsicParams, RpcClient};

pub const ENCOINTER_SCHEDULER: &str = "EncointerScheduler";
pub const COMMUNITIES: &str = "EncointerCommunities";

/// ApiClient extension that contains some convenience specific methods for encointer.
pub trait EncointerApi {
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
