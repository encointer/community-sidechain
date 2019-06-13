/*
	Copyright 2019 Supercomputing Systems AG

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
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;

use crypto::blake2s::Blake2s;
use log::*;
use my_node_runtime::Hash;
use sgx_crypto_helper::rsa3072::Rsa3072KeyPair;
use sgx_crypto_helper::RsaKeyPair;
use sgx_types::sgx_status_t;

use constants::RSA3072_SEALED_KEY_FILE;
use std::io::{Read, Write};
use std::sgxfs::SgxFile;
use std::vec::Vec;

pub fn read_rsa_keypair(status: &mut sgx_status_t) -> Rsa3072KeyPair {
	let mut keyvec: Vec<u8> = Vec::new();
	*status = read_file(&mut keyvec, RSA3072_SEALED_KEY_FILE);
	let key_json_str = std::str::from_utf8(&keyvec).unwrap();
	serde_json::from_str(&key_json_str).unwrap()
}

pub fn write_file(bytes: &[u8], filepath: &str) -> sgx_status_t {
	match SgxFile::create(filepath) {
		Ok(mut f) => match f.write_all(bytes) {
			Ok(()) => {
				info!("[Enclave] Writing keyfile '{}' successful", filepath);
				sgx_status_t::SGX_SUCCESS
			}
			Err(x) => {
				error!("[Enclave -] Writing keyfile '{}' failed! {}", filepath, x);
				sgx_status_t::SGX_ERROR_UNEXPECTED
			}
		},
		Err(x) => {
			error!("[Enclave !] Creating keyfile '{}' error! {}", filepath, x);
			sgx_status_t::SGX_ERROR_UNEXPECTED
		}
	}
}

pub fn read_file(mut keyvec: &mut Vec<u8>, filepath: &str) -> sgx_status_t {
	match SgxFile::open(filepath) {
		Ok(mut f) => match f.read_to_end(&mut keyvec) {
			Ok(len) => {
				info!("[Enclave] Read {} bytes from key file", len);
				sgx_status_t::SGX_SUCCESS
			}
			Err(x) => {
				error!("[Enclave] Read key file failed {}", x);
				sgx_status_t::SGX_ERROR_UNEXPECTED
			}
		},
		Err(x) => {
			error!("[Enclave] get_sealed_pcl_key cannot open key file, please check if key is provisioned successfully! {}", x);
			sgx_status_t::SGX_ERROR_UNEXPECTED
		}
	}
}

// FIXME: think about how statevec should be handled in case no counter exist such that we
// only need one read function. Maybe search and init COUNTERSTATE file upon enclave init?
pub fn read_counterstate(mut state_vec: &mut Vec<u8>, filepath: &str) -> sgx_status_t {
	match SgxFile::open(filepath) {
		Ok(mut f) => match f.read_to_end(&mut state_vec) {
			Ok(len) => {
				info!("[Enclave] Read {} bytes from counter file", len);
				sgx_status_t::SGX_SUCCESS
			}
			Err(x) => {
				error!("[Enclave] Read counter file failed {}", x);
				sgx_status_t::SGX_ERROR_UNEXPECTED
			}
		},
		Err(x) => {
			error!("[Enclave] can't get counter file! {}", x);
			state_vec.push(0);
			sgx_status_t::SGX_SUCCESS
		}
	}
}

pub fn decode_payload(ciphertext_slice: &[u8], rsa_pair: &Rsa3072KeyPair) -> Vec<u8> {
	let mut decrypted_buffer = Vec::new();
	rsa_pair.decrypt_buffer(ciphertext_slice, &mut decrypted_buffer).unwrap();
	decrypted_buffer
}

pub fn hash_from_slice(hash_slize: &[u8]) -> Hash {
	let mut g = [0; 32];
	g.copy_from_slice(&hash_slize[..]);
	Hash::from(&mut g)
}

pub fn blake2s(plaintext: &[u8]) -> [u8; 32] {
	let mut call_hash: [u8; 32] = Default::default();
	Blake2s::blake2s(&mut call_hash, &plaintext[..], &[0; 32]);
	call_hash
}
