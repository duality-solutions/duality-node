// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

//! Substrate CLI
mod cli;
mod command;

use color_eyre::eyre;
use std::{error, fmt};

pub use crate::cli::*;
pub use crate::command::*;
pub use sc_cli::{Error as NodeError, Result};

/// A helper to satisfy the requirements of `eyre`
/// compatible errors, which require `Send + Sync`
/// which are not satisfied by the `sp_*` crates.
#[derive(Debug)]
struct ErrorWrapper(std::sync::Arc<NodeError>);

// nothing is going to be sent to another thread
// it merely exists to glue two distinct error
// types together where the requirements differ
// with `Sync + Send` and without them for `wasm`.
unsafe impl Sync for ErrorWrapper {}
unsafe impl Send for ErrorWrapper {}

impl error::Error for ErrorWrapper {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		(&*self.0).source().and_then(|e| e.source())
	}
	fn description(&self) -> &str {
		"Error Wrapper"
	}
}

impl fmt::Display for ErrorWrapper {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", &*self.0)
	}
}

fn main() -> eyre::Result<()> {
	color_eyre::install()?;
	command::run().map_err(|e| ErrorWrapper(std::sync::Arc::new(e)))?;
	Ok(())
}
