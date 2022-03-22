// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

use sc_consensus::LongestChain;
use sc_consensus_aura::{ImportQueueParams, SlotProportion, StartAuraParams};
use sc_client_api::ExecutorProvider;
use sc_executor::NativeElseWasmExecutor;
use sc_finality_grandpa::GrandpaBlockImport;
use sc_network::NetworkService;
use sc_service::{
	Arc, Configuration, Error as ServiceError, KeystoreContainer,
	TaskManager, TFullBackend, TFullClient
};
use sc_telemetry::TelemetryHandle;

use sp_consensus::{
	CanAuthorWithNativeVersion, SlotData
};
use sp_consensus_aura::{
	ed25519::AuthorityPair, inherents::InherentDataProvider as AuraInherentDataProvider
};
use sp_runtime::{
	traits::{Block as BlockT}
};
use sp_timestamp::InherentDataProvider;

use substrate_prometheus_endpoint::Registry;

use duality_runtime::template::RuntimeApi;
use duality_primitives::Block;

type FullClient =
	TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;
type FullBackend = TFullBackend<Block>;
type FullSelectChain = LongestChain<FullBackend, Block>;
type FullGrandpaBlockImport = GrandpaBlockImport<
	FullBackend,
	Block,
	FullClient,
	FullSelectChain,
>;

// Our native executor instance.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
	/// Only enable the benchmarking host functions when we actually want to benchmark.
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	/// Otherwise we only use the default Substrate host functions.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		duality_runtime::template::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		duality_runtime::template::native_version()
	}
}

pub fn import_queue_builder(
	client: Arc<FullClient>,
	config: &Configuration,
	grandpa_block_import: FullGrandpaBlockImport,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<
	sc_consensus::DefaultImportQueue<
		Block,
		FullClient,
	>,
	ServiceError,
> {
	let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

	sc_consensus_aura::import_queue::<
		AuthorityPair,
		_,
		_,
		_,
		_,
		_,
		_
	>(ImportQueueParams {
		block_import: grandpa_block_import.clone(),
		justification_import: Some(Box::new(grandpa_block_import.clone())),
		client: client.clone(),
		create_inherent_data_providers: move |_, ()| async move {
			let timestamp = InherentDataProvider::from_system_time();
			let slot = AuraInherentDataProvider::from_timestamp_and_duration(
					*timestamp,
					slot_duration.slot_duration(),
				);

			Ok((timestamp, slot))
		},
		spawner: &task_manager.spawn_essential_handle(),
		can_author_with: CanAuthorWithNativeVersion::new(
			client.executor().clone(),
		),
		registry: config.prometheus_registry(),
		check_for_equivocation: Default::default(),
		telemetry: telemetry,
	}).map_err(Into::into)
}

pub fn block_author_builder(
	block_import: FullGrandpaBlockImport,
	client: Arc<FullClient>,
	force_authoring: bool,
	keystore: &KeystoreContainer,
	network: Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
	select_chain: FullSelectChain,
	telemetry: Option<TelemetryHandle>,
	transaction_pool: Arc<sc_transaction_pool::FullPool<Block, FullClient>>,
	task_manager: &TaskManager,
	prometheus_registry: Option<&Registry>
) -> Result<impl core::future::Future<Output = ()>, ServiceError>
{
	let backoff_authoring_blocks: Option<()> = None;
	let can_author_with = CanAuthorWithNativeVersion::new(client.executor().clone());
	let proposer_factory = sc_basic_authorship::ProposerFactory::new(
		task_manager.spawn_handle(),
		client.clone(),
		transaction_pool,
		prometheus_registry,
		telemetry.clone(),
	);
	let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

	sc_consensus_aura::start_aura::<
		AuthorityPair,
		_,
		_,
		_,
		_,
		_,
		_,
		_,
		_,
		_,
		_,
		_
	>(
		StartAuraParams {
			slot_duration,
			client: client.clone(),
			select_chain,
			block_import,
			proposer_factory,
			create_inherent_data_providers: move |_, ()| async move {
				let timestamp = InherentDataProvider::from_system_time();
				let slot = AuraInherentDataProvider::from_timestamp_and_duration(
						*timestamp,
						slot_duration.slot_duration(),
					);

				Ok((timestamp, slot))
			},
			force_authoring,
			backoff_authoring_blocks: backoff_authoring_blocks,
			keystore: keystore.sync_keystore(),
			can_author_with,
			sync_oracle: network.clone(),
			justification_sync_link: network.clone(),
			block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
			max_block_proposal_slot_portion: None,
			telemetry: telemetry.clone(),
		},
	).map_err::<ServiceError, _>(Into::into)
}
