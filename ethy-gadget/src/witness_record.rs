// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd. and Annie Lai Investment Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use benson_primitives::eth::{
	crypto::{AuthorityId, AuthoritySignature as Signature},
	EventId, Witness,
};
use log::{error, trace};
use std::collections::HashMap;

/// Tracks live witnesses
///
/// Stores witnesses per message event_id and digest
/// event_id -> digest -> [](authority, signature)
/// this structure allows resiliency incase different digests are witnessed, maliciously or not.
#[derive(Default)]
pub struct WitnessRecord {
	record: HashMap<EventId, HashMap<[u8; 32], Vec<(usize, Signature)>>>,
	/// local record of info of an event (tag (optional), block number of event)
	event_meta: HashMap<EventId, ([u8; 32], Option<Vec<u8>>)>,
	has_voted: HashMap<EventId, Vec<AuthorityId>>,
	/// `validators` - The ECDSA public (session) keys of validators ORDERED!
	validators: Vec<AuthorityId>,
}

impl WitnessRecord {
	/// Set the validator keys
	pub fn set_validators(&mut self, validators: Vec<AuthorityId>) {
		self.validators = validators;
	}
	/// Remove a witness record from memory
	pub fn clear(&mut self, event_id: EventId) {
		self.record.remove(&event_id);
		self.event_meta.remove(&event_id);
	}
	/// Return all known signatures for the witness on (event_id, digest)
	pub fn signatures_for(&self, event_id: EventId, digest: &[u8; 32]) -> Vec<Signature> {
		let mut signatures = vec![Signature::default(); self.validators.len()];
		let proofs = self.record.get(&event_id).unwrap().get(digest).unwrap();
		for (idx, signature) in proofs.into_iter() {
			let _ = std::mem::replace(&mut signatures[*idx], signature.clone());
		}
		signatures
	}
	/// Does the event identified by `event_id` `digest` have >= `threshold` support
	pub fn has_consensus(&self, event_id: EventId, digest: &[u8; 32], threshold: usize) -> bool {
		trace!(target: "ethy", "💎 event {:?}, records: {:?}", event_id, self.record.get(&event_id));
		let maybe_count = self.record.get(&event_id).and_then(|x| x.get(digest)).map(|v| v.len());

		trace!(target: "ethy", "💎 event {:?}, has # support: {:?}", event_id, maybe_count);
		maybe_count.unwrap_or_default() >= threshold
	}
	/// Return event metadata (block, optional tag)
	pub fn event_metadata(&self, event_id: EventId) -> Option<&([u8; 32], Option<Vec<u8>>)> {
		self.event_meta.get(&event_id)
	}
	/// Note event metadata
	pub fn note_event_metadata(&mut self, event_id: EventId, block: [u8; 32], tag: Option<Vec<u8>>) {
		self.event_meta.entry(event_id).or_insert((block, tag));
	}
	/// Note a witness if we haven't seen it before
	/// Returns true if the witness was noted, i.e previously unseen
	pub fn note(&mut self, witness: &Witness) -> bool {
		if self
			.has_voted
			.get(&witness.event_id)
			.map(|votes| votes.binary_search(&witness.authority_id).is_ok())
			.unwrap_or_default()
		{
			trace!(target: "ethy", "💎 witness previously seen: {:?}", witness.event_id);
			return false;
		}

		// Convert authority ECDSA public key into ordered index
		// this is useful to efficiently generate a proof later
		let validators = self.validators.clone();
		let authority_to_index = || -> Option<usize> {
			let maybe_pos = validators.iter().position(|v| v == &witness.authority_id);
			if maybe_pos.is_none() {
				// this implies the witness is not an active validator
				// this should not happen (i.e. the witness should be invalidated sooner in the lifecycle)
				error!(target: "ethy", "💎 unexpected authority witness. event: {:?}, authority: {:?}", witness.event_id, witness.authority_id);
			}
			maybe_pos
		};

		// Spaghetti code to insert into nested map
		// There are 3 cases:
		// 1) first time observing an event
		// 2) known event, first time observing this digest
		// 3) known event & known digest, first time observing this witness
		// all of this to ensure we have consensus over the exact values
		self.record
			.entry(witness.event_id)
			.and_modify(|event_digests| {
				event_digests
					.entry(witness.digest)
					.and_modify(|signatures| {
						// case 2
						authority_to_index()
							.map(|authority_index| signatures.push((authority_index, witness.signature.clone())));
					})
					.or_insert({
						// case 3
						if let Some(authority_index) = authority_to_index() {
							vec![(authority_index, witness.signature.clone())]
						} else {
							// no authority index. should not happen, bail.
							return;
						}
					});
			})
			.or_insert({
				// case 1
				let mut digest_signatures = HashMap::<[u8; 32], Vec<(usize, Signature)>>::default();
				authority_to_index().map(|authority_index| {
					digest_signatures.insert(witness.digest, vec![(authority_index, witness.signature.clone())])
				});
				digest_signatures
			});
		trace!(target: "ethy", "💎 witness recorded: {:?}, {:?}", witness.event_id, witness.authority_id);

		// Mark authority as voted
		match self.has_voted.get_mut(&witness.event_id) {
			None => {
				// first vote for this event_id we've seen
				self.has_voted
					.insert(witness.event_id, vec![witness.authority_id.clone()]);
			}
			Some(votes) => {
				// subsequent vote for a known event_id
				if let Err(idx) = votes.binary_search(&witness.authority_id) {
					votes.insert(idx, witness.authority_id.clone());
				}
			}
		}

		return true;
	}
}
