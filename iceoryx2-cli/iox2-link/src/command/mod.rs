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

mod execute;
mod list;
mod paths;

pub(crate) use execute::*;
pub(crate) use list::*;
pub(crate) use paths::*;

/// Prefix for the tunnel backend.
pub(crate) const TUNNEL_BACKEND: &str = "iox2-link-tunnel-";

/// Prefix for the gateway backend.
pub(crate) const GATEWAY_BACKEND: &str = "iox2-link-gateway-";

/// Message printed when no tunnel CLIs are installed.
pub(crate) const TUNNEL_EMPTY: &str = "No tunnel CLIs found.\n\n\
    Install one to get started, e.g.:\n  \
    cargo install iceoryx2-integrations-zenoh-link-tunnel-cli";

/// Message printed when no gateway CLIs are installed.
pub(crate) const GATEWAY_EMPTY: &str = "No gateway CLIs found.";
