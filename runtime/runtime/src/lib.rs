// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

#![cfg_attr(not(feature = "std"), no_std)]

// `construct_runtime!` does a lot of recursion
// and requires us to increase the limit to 256.
#![recursion_limit = "256"]

#[cfg(feature = "with-template-runtime")]
pub mod template;
