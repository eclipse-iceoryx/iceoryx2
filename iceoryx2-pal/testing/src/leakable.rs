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

/// **Only for testing purposes!**
///
/// Marks types that can leak resources required for cleanup tests.
/// The system resource is leaked but the process local internal constructs
/// are still cleaned up properly. Those internal constructs could be:
///
/// * file descriptors
/// * memory mappings
/// * ...
pub trait Leakable {
    fn leak(self);
}
