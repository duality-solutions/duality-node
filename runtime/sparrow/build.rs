// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

use substrate_wasm_builder::WasmBuilder;

fn main() {
	WasmBuilder::new()
		.with_current_project()
		.export_heap_base()
		.import_memory()
		.build()
}
