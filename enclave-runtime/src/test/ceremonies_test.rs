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

pub fn test_register_bootstrapper() {
	// given
	let (_, mut state, shard, mrenclave, ..) = test_setup();
	let mut opaque_vec = Vec::new();

	// Create the sender account.
	let sender = funded_pair();

	//Update state
	//Create the community
	//Create a bootstrapper
	//Set phase registering

	//Call to register bootstrapper
	let trusted_call = TrustedCall::ceremonies_register_participant(sender_acc, cid, None).sign(
		&sender.clone().into(),
		0,
		&mrenclave,
		&shard,
	);

	//execute call

	//check
	/*
	let bootstrapper_count = state
		.execute_with(|| get_evm_account_storages(&execution_address, &H256::zero()))
		.unwrap();

	 */
}
