// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

/// Provides the recommended inter-process
/// [`Reactor`](crate::reactor::Reactor) concept
/// implementation for the target.
pub type Ipc = crate::reactor::posix_select::Reactor;

/// Provides the recommended process-local
/// [`Reactor`](crate::reactor::Reactor) concept
/// implementation for the target.
pub type Local = crate::reactor::posix_select::Reactor;
