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
#![cfg(feature = "test-doughnut")]

//! Doughnut/CENNZnut runtime validation tests

use benson_primitives::types::AccountId;
use benson_runtime::{impls::BensonDispatchVerifier, BensonDoughnut};
use cennznut::{self, CENNZnut, CENNZnutV0};
use codec::Encode;
use frame_support::{additional_traits::DelegatedDispatchVerifier, assert_err, assert_ok};
use sp_keyring::AccountKeyring;
use sp_runtime::{traits::DoughnutSigning, Doughnut, DoughnutV0};

fn test_issuer() -> [u8; 32] {
	AccountKeyring::Alice.to_raw_public()
}

fn test_holder() -> [u8; 32] {
	AccountKeyring::Bob.to_raw_public()
}

const CONTRACT_ADDRESS: [u8; 32] = [0x11_u8; 32];

// A helper to make test doughnuts
pub fn make_doughnut(domain: &str, domain_payload: Vec<u8>) -> BensonDoughnut {
	let mut doughnut_v0 = DoughnutV0 {
		holder: test_holder(),
		issuer: test_issuer(),
		domains: vec![(domain.to_string(), domain_payload)],
		expiry: 3000,
		not_before: 0,
		payload_version: 0,
		signature_version: 0,
		signature: Default::default(),
	};
	doughnut_v0
		.sign_sr25519(&AccountKeyring::Alice.pair().to_ed25519_bytes())
		.expect("it signs ok");
	let doughnut = Doughnut::V0(doughnut_v0);
	BensonDoughnut::new(doughnut)
}

fn verify_dispatch(doughnut: &BensonDoughnut, module: &str, method: &str) -> Result<(), &'static str> {
	<BensonDispatchVerifier as DelegatedDispatchVerifier>::verify_dispatch(doughnut, module, method, vec![])
}

fn verify_runtime_to_contract(
	caller: &AccountId,
	doughnut: &BensonDoughnut,
	contract_addr: &AccountId,
) -> Result<(), &'static str> {
	<BensonDispatchVerifier as DelegatedDispatchVerifier>::verify_runtime_to_contract_call(
		caller,
		doughnut,
		contract_addr,
	)
}

// A helper to make test CENNZnuts
pub fn make_runtime_cennznut(module: &str, method: &str) -> CENNZnut {
	let method_obj = cennznut::v0::method::Method {
		name: method.to_string(),
		block_cooldown: None,
		constraints: None,
	};
	let module_obj = cennznut::v0::module::Module {
		name: module.to_string(),
		block_cooldown: None,
		methods: vec![(method.to_string(), method_obj)],
	};
	CENNZnut::V0(CENNZnutV0 {
		modules: vec![(module.to_string(), module_obj)],
		contracts: Default::default(),
	})
}

// A helper to make test CENNZnuts
pub fn make_contract_cennznut(contract_addr: &AccountId) -> CENNZnut {
	let method_obj = cennznut::v0::method::Method {
		name: "call".to_string(),
		block_cooldown: None,
		constraints: None,
	};
	let module_obj = cennznut::v0::module::Module {
		name: "contracts".to_string(),
		block_cooldown: None,
		methods: vec![("call".to_string(), method_obj)],
	};
	let address = contract_addr.clone();
	let contract_obj = cennznut::v0::contract::Contract::new(&address.into());
	CENNZnut::V0(CENNZnutV0 {
		modules: vec![("contracts".to_string(), module_obj)],
		contracts: vec![(contract_obj.address, contract_obj)],
	})
}

#[test]
fn it_works() {
	let cennznut = make_runtime_cennznut("attestation", "attest");
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_ok!(verify_dispatch(&doughnut, "trml-attestation", "attest"));
}

#[test]
fn it_works_with_arbitrary_prefix_short() {
	let cennznut = make_runtime_cennznut("attestation", "attest");
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_ok!(verify_dispatch(&doughnut, "t-attestation", "attest"));
}

#[test]
fn it_works_with_arbitrary_prefix_long() {
	let cennznut = make_runtime_cennznut("attestation", "attest");
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_ok!(verify_dispatch(&doughnut, "trmlcrml-attestation", "attest"));
}

#[test]
fn it_works_with_arbitrary_non_ascii_characters() {
	let cennznut = make_runtime_cennznut("attestation", "attest");
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_ok!(verify_dispatch(&doughnut, "ṫřṁl-attestation", "attest"));
	assert_eq!("ṫřṁl-attestation".len(), 21);
	assert_eq!("ṫřṁl-attestation".chars().count(), 16);
}

#[test]
fn it_fails_when_not_using_the_benson_domain() {
	let doughnut = make_doughnut("test", Default::default());
	assert_err!(
		verify_dispatch(&doughnut, "trml-module", "method"),
		"CENNZnut does not grant permission for benson domain"
	);
}

#[test]
fn it_fails_with_bad_cennznut_encoding() {
	let doughnut = make_doughnut("benson", vec![1, 2, 3, 4, 5]);
	assert_err!(
		verify_dispatch(&doughnut, "trml-module", "method"),
		"Bad CENNZnut encoding"
	);
}

#[test]
fn it_fails_when_module_is_not_authorized() {
	let cennznut = make_runtime_cennznut("attestation", "attest");
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_err!(
		verify_dispatch(&doughnut, "trml-generic-asset", "attest"),
		"CENNZnut does not grant permission for module"
	);
}

#[test]
fn it_fails_when_method_is_not_authorized() {
	let cennznut = make_runtime_cennznut("attestation", "attest");
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_err!(
		verify_dispatch(&doughnut, "trml-attestation", "remove"),
		"CENNZnut does not grant permission for method"
	);
}

#[test]
fn it_fails_when_module_name_is_invalid() {
	let cennznut = make_runtime_cennznut("attestation", "attest");
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_err!(
		verify_dispatch(&doughnut, "trmlattestation", "attest"),
		"CENNZnut does not grant permission for module"
	);
}

#[test]
fn it_fails_when_prefix_is_empty() {
	let cennznut = make_runtime_cennznut("attestation", "attest");
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_err!(
		verify_dispatch(&doughnut, "-attestation", "attest"),
		"error during module name segmentation"
	);
}

#[test]
fn it_fails_when_module_name_is_empty() {
	let cennznut = make_runtime_cennznut("attestation", "attest");
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_err!(
		verify_dispatch(&doughnut, "trml-", "attest"),
		"error during module name segmentation"
	);
}

#[test]
fn it_fails_when_module_name_and_prefix_are_empty() {
	let cennznut = make_runtime_cennznut("attestation", "attest");
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_err!(
		verify_dispatch(&doughnut, "-", "attest"),
		"error during module name segmentation"
	);
}

#[test]
fn it_fails_when_using_contract_cennznut_for_runtime() {
	let cennznut = make_contract_cennznut(&CONTRACT_ADDRESS.into());
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_err!(
		verify_dispatch(&doughnut, "attestation", "attest"),
		"CENNZnut does not grant permission for module"
	);
}

#[test]
fn it_fails_runtime_to_contract_with_invalid_domain() {
	let cennznut = make_contract_cennznut(&CONTRACT_ADDRESS.into());
	let doughnut = make_doughnut("sendsnet", cennznut.encode());
	assert_err!(
		verify_runtime_to_contract(&test_issuer().into(), &doughnut, &CONTRACT_ADDRESS.into()),
		"CENNZnut does not grant permission for benson domain"
	);
}

#[test]
fn it_fails_runtime_to_contract_with_invalid_contract() {
	let cennznut = make_contract_cennznut(&CONTRACT_ADDRESS.into());
	let doughnut = make_doughnut("benson", cennznut.encode());
	let unregistered_contract: [u8; 32] = [0x22; 32];
	assert_err!(
		verify_runtime_to_contract(&test_issuer().into(), &doughnut, &unregistered_contract.into()),
		"CENNZnut does not grant permission for contract"
	);
}

#[test]
fn it_succeeds_runtime_to_contract_with_valid_contract() {
	let cennznut = make_contract_cennznut(&CONTRACT_ADDRESS.into());
	let doughnut = make_doughnut("benson", cennznut.encode());
	assert_ok!(verify_runtime_to_contract(
		&test_issuer().into(),
		&doughnut,
		&CONTRACT_ADDRESS.into()
	));
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Invalid doughnut caller")]
fn it_fails_runtime_to_contract_with_invalid_caller() {
	let cennznut = make_contract_cennznut(&CONTRACT_ADDRESS.into());
	let doughnut = make_doughnut("benson", cennznut.encode());
	let invalid_caller: [u8; 32] = [0x55; 32];
	let _ = verify_runtime_to_contract(&invalid_caller.into(), &doughnut, &CONTRACT_ADDRESS.into());
}
