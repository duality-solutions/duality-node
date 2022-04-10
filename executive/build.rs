// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

use cfg_aliases::cfg_aliases;
use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {
	cfg_aliases! {
		template: { feature = "with-template-runtime" },
	}

	generate_cargo_keys();
	rerun_if_git_head_changed();
}
