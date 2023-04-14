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
use codec::Decode;
use encointer_primitives::balances::BalanceType;
use log::*;

pub fn decode_encointer_balance(maybe_encoded_balance: Option<Vec<u8>>) -> Option<BalanceType> {
	maybe_encoded_balance.and_then(|encoded_balance| {
		if let Ok(vd) = BalanceType::decode(&mut encoded_balance.as_slice()) {
			Some(vd)
		} else {
			error!("Could not decode balance. maybe hasn't been set? {:x?}", encoded_balance);
			None
		}
	})
}
