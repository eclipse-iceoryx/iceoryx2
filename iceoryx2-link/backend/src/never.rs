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

/// A type without values, so code over it can never run.
///
/// Fills an associated type an implementer has nothing for, e.g. the relay
/// of a pattern a backend does not bridge.
// TODO: replace with the never type `!` once it is stablized.
#[derive(Debug, Clone, Copy)]
pub enum Never {}
