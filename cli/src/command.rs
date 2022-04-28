// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2019-2021 PureStake Inc.
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

use cfg_if::cfg_if;
use crate::{
	cli::{Cli, Subcommand},
};
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};

#[cfg(template)]
use duality_executive::template::{
	chain_spec as template_chain,
	executive as executive_template
};

#[cfg(sparrow)]
use duality_executive::sparrow::{
	chain_spec as sparrow_chain,
	executive as executive_sparrow
};

use duality_service::IdentifyVariant;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Substrate Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"support.anonymous.an".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		cfg_if! {
			if #[cfg(sparrow)] {
				Ok(match id {
					"dev" => Box::new(sparrow_chain::development_config()?),
					"" | "local" => Box::new(sparrow_chain::local_testnet_config()?),
					path =>
						Box::new(sparrow_chain::ChainSpec::from_json_file(std::path::PathBuf::from(path))?),
				})
			} else if #[cfg(template)] {
				Ok(match id {
					"dev" => Box::new(template_chain::development_config()?),
					"" | "local" => Box::new(template_chain::local_testnet_config()?),
					path =>
					Box::new(template_chain::ChainSpec::from_json_file(std::path::PathBuf::from(path))?),
				})
			} else {
				Err("No runtime is included in this build")
			}
		}
	}

	fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		cfg_if! {
			if #[cfg(sparrow)] {
				&runtime_sparrow::VERSION
			} else if #[cfg(template)] {
				&runtime_template::VERSION
			} else {
				// TODO: allow disabled runtime support
			}
		}
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),

		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				cmd.run(config.chain_spec, config.network)
			})
		},

		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = duality_service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}

		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = duality_service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}

		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = duality_service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}

		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = duality_service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}

		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				cmd.run(config.database)
			})
		},

		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, backend, _, task_manager) = duality_service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, backend), task_manager))
			})
		}

		Some(Subcommand::Benchmark(cmd)) => {
			if cfg!(feature = "runtime-benchmarks") {
				let runner = cli.create_runner(cmd)?;
				let chain_spec = &runner.config().chain_spec;
				match chain_spec {
					#[cfg(template)]
					spec if spec.is_template() => {
						return runner.sync_run(|config| {
							cmd.run::<runtime_template::Block, executive_template::ExecutorDispatch>(
								config
							)
						})
					},
					#[cfg(sparrow)]
					spec if spec.is_sparrow() => {
						return runner.sync_run(|config| {
							cmd.run::<runtime_sparrow::Block, executive_sparrow::ExecutorDispatch>(
								config
							)
						})
					},
					_ => panic!("invalid chain spec"),
				}
			} else {
				Err("Benchmarking wasn't enabled when building the node. You can enable it with \
				     `--features runtime-benchmarks`."
					.into())
			}
		},

		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| async move {
				duality_service::new_full(config).map_err(sc_cli::Error::Service)
			})
		},
	}
}
