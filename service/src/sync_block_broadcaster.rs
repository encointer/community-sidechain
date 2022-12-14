/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG
	Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.

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

#[cfg(test)]
use mockall::predicate::*;
#[cfg(test)]
use mockall::*;

use crate::{
	globals::tokio_handle::GetTokioHandle,
	worker::{AsyncBlockBroadcaster, WorkerResult},
};
use its_primitives::types::block::SignedBlock as SignedSidechainBlock;
use std::sync::Arc;

/// Allows to broadcast blocks, does it in a synchronous (i.e. blocking) manner
#[cfg_attr(test, automock)]
pub trait BroadcastBlocks {
	fn broadcast_blocks(&self, blocks: Vec<SignedSidechainBlock>) -> WorkerResult<()>;
}

pub struct SyncBlockBroadcaster<T, W> {
	tokio_handle: Arc<T>,
	worker: Arc<W>,
}

impl<T, W> SyncBlockBroadcaster<T, W> {
	pub fn new(tokio_handle: Arc<T>, worker: Arc<W>) -> Self {
		SyncBlockBroadcaster { tokio_handle, worker }
	}
}

impl<T, W> BroadcastBlocks for SyncBlockBroadcaster<T, W>
where
	T: GetTokioHandle,
	W: AsyncBlockBroadcaster,
{
	fn broadcast_blocks(&self, blocks: Vec<SignedSidechainBlock>) -> WorkerResult<()> {
		let handle = self.tokio_handle.get_handle();
		handle.block_on(self.worker.broadcast_blocks(blocks))
	}
}
