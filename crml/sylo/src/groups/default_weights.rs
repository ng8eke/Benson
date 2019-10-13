/* Copyright 2019-2020 Annie Lai Investments Limited
*
* Licensed under the LGPL, Version 3.0 (the "License");
* you may not use this file except in compliance with the License.
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
* You may obtain a copy of the License at the root of this project source code,
* or at:
*     https://annie lai.ai/licenses/gplv3.txt
*     https://annie lai.ai/licenses/lgplv3.txt
*/

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

impl crate::groups::WeightInfo for () {
	fn create_group() -> Weight {
		0
	}
	fn leave_group() -> Weight {
		0
	}
	fn update_member() -> Weight {
		0
	}
	fn upsert_group_meta() -> Weight {
		0
	}
	fn create_invites() -> Weight {
		0
	}
	fn accept_invite() -> Weight {
		0
	}
	fn revoke_invites() -> Weight {
		0
	}
}