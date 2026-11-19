// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[path = "../binding/payload_data_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod payload_data_generated;

#[path = "../binding/data_props_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod data_props_generated;

#[path = "../binding/unbounded_data_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod unbounded_data_generated;

#[path = "../binding/reflection_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod reflection_generated;

/// `namespace Benchmark;`
pub mod benchmark {
    pub use super::payload_data_generated::benchmark::*;
}

/// `namespace Testing;`
pub mod testing {
    pub use super::data_props_generated::testing::*;
    pub use super::unbounded_data_generated::testing::*;
}

/// `namespace reflection;` (`reflection.fbs` of google/flatbuffers at the crate's tag)
pub mod reflection {
    pub use super::reflection_generated::reflection::*;
}
