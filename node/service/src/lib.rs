// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2019-2021 PureStake Inc.
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

mod client;
use client::Client;

pub use sc_executor::NativeElseWasmExecutor;

use core::future::Future;
use crate::client::RuntimeApiCollection;

#[cfg(feature = "with-template-runtime")]
use duality_executive::template::executive as template_executive;

use sc_client_api::StateBackendFor;
use sc_consensus::{
	BasicQueue, DefaultImportQueue, LongestChain
};
use sc_finality_grandpa::{GrandpaBlockImport, SharedVoterState};
use sc_network::NetworkService;
use sc_service::{
	BuildNetworkParams, ChainSpec, Configuration, error::Error as ServiceError,
	KeystoreContainer, NativeExecutionDispatch, PartialComponents, TaskManager,
	TFullClient, TFullBackend
};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker};

use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use sp_trie::PrefixedMemoryDB;
use sp_api::ConstructRuntimeApi;

use runtime_primitives::Block;

use std::{sync::Arc, time::Duration};

type FullClient<RuntimeApi, Executor> =
	TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;
type FullBackend = TFullBackend<Block>;
type FullSelectChain = LongestChain<FullBackend, Block>;
type FullGrandpaBlockImport<RuntimeApi, Executor> = GrandpaBlockImport<
	FullBackend,
	Block,
	FullClient<RuntimeApi, Executor>,
	FullSelectChain,
>;

/// Can be called for a `Configuration` to check if it is a configuration for
/// the network.
pub trait IdentifyVariant {
	/// Returns `true` if this is a configuration for a template network.
	fn is_template(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_template(&self) -> bool {
		self.id().ends_with("template")
	}
}

/// Builds a new object suitable for chain operations.
pub fn new_chain_ops(
	mut config: &mut Configuration,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
> {
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	if config.chain_spec.is_template() {
		let PartialComponents {
			client,
			backend,
			import_queue,
			task_manager,
			..
		} = new_partial::<template_runtime::RuntimeApi, template_executive::ExecutorDispatch, _>(
			config,
			template_executive::import_queue_builder
		)?;
		Ok((
			Arc::new(Client::Template(client)),
			backend,
			import_queue,
			task_manager,
		))
	} else {
		let PartialComponents {
			client,
			backend,
			import_queue	,
			task_manager,
			..
		} = new_partial::<template_runtime::RuntimeApi, template_executive::ExecutorDispatch, _>(
			config,
			template_executive::import_queue_builder
		)?;
		Ok((
			Arc::new(Client::Template(client)),
			backend,
			import_queue,
			task_manager,
		))
	}
}

pub fn new_partial<RuntimeApi, Executor, ImportQueueBuilder>(
	config: &Configuration,
	import_queue_builder: ImportQueueBuilder,
) -> Result<
	sc_service::PartialComponents<
		FullClient<RuntimeApi, Executor>,
		FullBackend,
		FullSelectChain,
		DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
		sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
		(
			FullGrandpaBlockImport<RuntimeApi, Executor>,
			sc_finality_grandpa::LinkHalf<Block, FullClient<RuntimeApi, Executor>, FullSelectChain>,
			Option<Telemetry>,
		),
	>,
	ServiceError,
> where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
	ImportQueueBuilder: FnOnce(
		Arc<FullClient<RuntimeApi, Executor>>,
		&Configuration,
		FullGrandpaBlockImport<RuntimeApi, Executor>,
		Option<TelemetryHandle>,
		&TaskManager,
	) -> Result<
		DefaultImportQueue<
			Block,
			FullClient<RuntimeApi, Executor>,
		>,
		ServiceError,
	>,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = NativeElseWasmExecutor::<Executor>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
	);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			&config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
		client.clone(),
		&(client.clone() as Arc<_>),
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;

	let import_queue = import_queue_builder(
		client.clone(),
		config,
		grandpa_block_import.clone(),
		telemetry.as_ref().map(|telemetry| telemetry.handle()),
		&task_manager,
	)?;

	Ok(PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (grandpa_block_import, grandpa_link, telemetry),
	})
}

pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
	// FIXME: Should be substituted by a non-optional base runtime!
	#[cfg(feature = "with-template-runtime")]
	new_node::<
		template_runtime::RuntimeApi,
		template_executive::ExecutorDispatch,
		_,
		_,
		_,
	>(
		config,
		template_executive::import_queue_builder,
		template_executive::block_author_builder
	).map_err(Into::into)
}

/// Builds a new instance for a full client.
pub fn new_node<RuntimeApi, Executor, ImportQueueBuilder, BlockAuthorBuilder, CoreFuture>(
	mut config: Configuration,
	import_queue_builder :ImportQueueBuilder,
	block_authoring_builder: BlockAuthorBuilder
) -> Result<TaskManager, ServiceError>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
	ImportQueueBuilder: FnOnce(
		Arc<FullClient<RuntimeApi, Executor>>,
		&Configuration,
		FullGrandpaBlockImport<RuntimeApi, Executor>,
		Option<TelemetryHandle>,
		&TaskManager,
	) -> Result<
		DefaultImportQueue<
			Block,
			FullClient<RuntimeApi, Executor>,
		>,
		ServiceError,
	>,
	BlockAuthorBuilder: FnOnce(
		FullGrandpaBlockImport<RuntimeApi, Executor>,
		Arc<FullClient<RuntimeApi, Executor>>,
		bool,
		&KeystoreContainer,
		Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
		FullSelectChain,
		Option<TelemetryHandle>,
		Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>>,
		&TaskManager,
		Option<&substrate_prometheus_endpoint::Registry>
	) -> Result<CoreFuture, ServiceError>,
	CoreFuture: Future<Output = ()> + Send + 'static
{
	let PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (block_import, grandpa_link, mut telemetry),
	} = new_partial::<RuntimeApi, Executor, ImportQueueBuilder>(&config, import_queue_builder)?;

	config.network.extra_sets.push(sc_finality_grandpa::grandpa_peers_set_config());
	let warp_sync = Arc::new(sc_finality_grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		grandpa_link.shared_authority_set().clone(),
		Vec::default(),
	));

	let (network, system_rpc_tx, network_starter) =
		sc_service::build_network(BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync: Some(warp_sync),
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps =
				duality_rpc::FullDeps {
					client: client.clone(),
					pool: pool.clone(),
					deny_unsafe
				};
			duality_rpc::create_full(deps).map_err(Into::into)
		})
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_extensions_builder,
		backend,
		system_rpc_tx,
		config,
		telemetry: telemetry.as_mut(),
	})?;

	if role.is_authority() {
		let block_author = block_authoring_builder(
			block_import,
			client.clone(),
			force_authoring,
			&keystore_container,
			network.clone(),
			select_chain,
			telemetry.as_ref().map(|telemetry| telemetry.handle()),
			transaction_pool,
			&task_manager,
			prometheus_registry.as_ref()
		);

		// the authoring task is considered essential, i.e. if it
		// fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking("block-authoring", Some("block-authoring"), block_author?);
	}

	// if the node isn't actively participating in consensus then it doesn't
	// need a keystore, regardless of which protocol we use below.
	let keystore =
		if role.is_authority() { Some(keystore_container.sync_keystore()) } else { None };

	let grandpa_config = sc_finality_grandpa::Config {
		// FIXME #1578 make this available through chainspec
		gossip_duration: Duration::from_millis(333),
		justification_period: 512,
		name: Some(name),
		observer_enabled: false,
		keystore,
		local_role: role,
		telemetry: telemetry.as_ref().map(|x| x.handle()),
	};

	if enable_grandpa {
		// start the full GRANDPA voter
		// NOTE: non-authorities could run the GRANDPA observer protocol, but at
		// this point the full voter should provide better guarantees of block
		// and vote data availability than the observer. The observer has not
		// been tested extensively yet and having most nodes in a network run it
		// could lead to finality stalls.
		let grandpa_config = sc_finality_grandpa::GrandpaParams {
			config: grandpa_config,
			link: grandpa_link,
			network,
			voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state: SharedVoterState::empty(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
		};

		// the GRANDPA voter task is considered infallible, i.e.
		// if it fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	network_starter.start_network();
	Ok(task_manager)
}
