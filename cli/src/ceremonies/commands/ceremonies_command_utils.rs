//todo: add license

use codec::{Decode, Encode};
use encointer_primitives::{
	ceremonies::ProofOfAttendance,
	communities::{CommunityIdentifier, GeoHash},
	scheduler::CeremonyIndexType,
};
use ita_stf::{AccountId, Signature};
use log::*;
use sp_core::{sr25519 as sr25519_core, Pair};

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
		prover_public: AccountId::from(prover.clone()),
		ceremony_index: cindex,
		community_identifier: cid,
		attendee_public: AccountId::from(sr25519_core::Public::from(attendee.public())),
		attendee_signature: Signature::from(sr25519_core::Signature::from(
			attendee.sign(&msg.encode()),
		)),
	}
}

pub fn get_geo_hash_from_str(geo_hash: &str) -> GeoHash {
	GeoHash::try_from(geo_hash).unwrap()
}
