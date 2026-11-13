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

/// `namespace Benchmark;`
pub mod benchmark {
    /// Root type `Benchmark.PayloadData`.
    pub const PAYLOAD_DATA: &[u8] = include_bytes!("../schema/payload_data.bfbs");
}

/// `namespace Testing;`
pub mod testing {
    /// Root type `Testing.BoundedData`.
    pub const BOUNDED_DATA: &[u8] = include_bytes!("../schema/bounded_data.bfbs");
    /// Root type `Testing.IncompatibleData`.
    pub const INCOMPATIBLE_DATA: &[u8] = include_bytes!("../schema/incompatible_data.bfbs");
    /// Root type `Testing.UnboundedData`.
    pub const UNBOUNDED_DATA: &[u8] = include_bytes!("../schema/unbounded_data.bfbs");
    /// Root type `Testing.DataProps`.
    pub const DATA_PROPS: &[u8] = include_bytes!("../schema/data_props.bfbs");
}
