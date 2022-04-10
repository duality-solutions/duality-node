// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

use sc_consensus::LongestChain;
use sc_consensus_babe::{
    Config as BabeConfig, BabeParams, block_import, import_queue,
    SlotProportion, start_babe
};
use sc_client_api::ExecutorProvider;
use sc_executor::NativeElseWasmExecutor;
use sc_finality_grandpa::GrandpaBlockImport;
use sc_network::NetworkService;
use sc_service::{
	Arc, Configuration, Error as ServiceError, KeystoreContainer,
	TaskManager, TFullBackend, TFullClient
};
use sc_telemetry::TelemetryHandle;

use sp_consensus::CanAuthorWithNativeVersion;
use sp_runtime::{
	traits::{Block as BlockT}
};

use substrate_prometheus_endpoint::Registry;

use runtime_sparrow::RuntimeApi;
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
		runtime_sparrow::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		runtime_sparrow::native_version()
	}
}

pub fn import_queue_builder(
	client: Arc<FullClient>,
	config: &Configuration,
	grandpa_block_import: FullGrandpaBlockImport,
    select_chain: FullSelectChain,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<
	sc_consensus::DefaultImportQueue<
		Block,
		FullClient,
	>,
	ServiceError,
> {
	let justification_import = grandpa_block_import.clone();
	let babe_config = BabeConfig::get_or_compute(&*client)?;
	let (block_import, babe_link) =
		block_import(babe_config.clone(), grandpa_block_import, client.clone())?;
	let slot_duration = babe_link.config().slot_duration();

	import_queue(
		babe_link.clone(),
		block_import.clone(),
		Some(Box::new(justification_import)),
		client.clone(),
		select_chain.clone(),
		move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

			let slot =
				sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_duration(
					*timestamp,
					slot_duration,
				);

			Ok((timestamp, slot))
		},
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		CanAuthorWithNativeVersion::new(client.executor().clone()),
		telemetry,
	).map_err::<ServiceError, _>(Into::into)
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
	// TODO: replace with something less hacky
	// start hacky section
	let babe_config_tmp = BabeConfig::get_or_compute(&*client)?;
	let (_block_import_tmp, babe_link) =
	sc_consensus_babe::block_import(babe_config_tmp.clone(), block_import.clone(), client.clone())?;
	let slot_duration = babe_link.config().slot_duration();
	// end hacky section
	let babe_config = BabeParams {
		keystore: keystore.sync_keystore(),
		client: client.clone(),
		select_chain,
		block_import,
		env: proposer_factory,
		sync_oracle: network.clone(),
		justification_sync_link: network.clone(),
		create_inherent_data_providers: move |parent, ()| {
			let client_clone = client.clone();

			async move {
				let uncles = sc_consensus_uncles::create_uncles_inherent_data_provider(
					&*client_clone,
					parent,
				)?;

				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

				let slot =
					sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_duration(
						*timestamp,
						slot_duration,
					);

				Ok((timestamp, slot, uncles))
			}
		},
		force_authoring,
		backoff_authoring_blocks,
		babe_link,
		can_author_with,
		block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
		max_block_proposal_slot_portion: None,
		telemetry: telemetry,
	};

	start_babe(babe_config).map_err::<ServiceError, _>(Into::into)
}
