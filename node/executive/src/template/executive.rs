// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

use sc_consensus_aura::{ImportQueueParams};
use sc_client_api::ExecutorProvider;
use sc_executor::NativeElseWasmExecutor;

use sp_consensus::SlotData;
use sp_consensus_aura::ed25519::AuthorityPair;

use template_runtime::RuntimeApi;
use runtime_primitives::Block;

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
		template_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		template_runtime::native_version()
	}
}

pub fn import_queue_builder(
	client: sc_service::Arc<sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>>,
	config: &sc_service::Configuration,
	grandpa_block_import: sc_finality_grandpa::GrandpaBlockImport<
		sc_service::TFullBackend<Block>,
		Block,
		sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>,
		sc_consensus::LongestChain<sc_service::TFullBackend<Block>, Block>,
	>,
	telemetry: Option<sc_telemetry::TelemetryHandle>,
	task_manager: &sc_service::TaskManager,
) -> Result<
	sc_consensus::DefaultImportQueue<
		Block,
		sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>,
	>,
	sc_service::Error,
> {
	let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

	sc_consensus_aura::import_queue::<AuthorityPair, _, _, _, _, _, _>(ImportQueueParams {
		block_import: grandpa_block_import.clone(),
		justification_import: Some(Box::new(grandpa_block_import.clone())),
		client: client.clone(),
		create_inherent_data_providers: move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

			let slot =
				sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
					*timestamp,
					slot_duration.slot_duration(),
				);

			Ok((timestamp, slot))
		},
		spawner: &task_manager.spawn_essential_handle(),
		can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(
			client.executor().clone(),
		),
		registry: config.prometheus_registry(),
		check_for_equivocation: Default::default(),
		telemetry: telemetry,
	}).map_err(Into::into)
}