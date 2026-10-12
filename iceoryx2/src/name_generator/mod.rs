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

use core::fmt::Debug;

use crate::{config::Config, node::node_name::NodeName, port::port_name::PortName};

pub mod default_name_generator;

/// Describes failures that can occur when a [`NameGenerator`] is opened or created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameGeneratorOpenOrCreateError {
    /// Insufficient permissions to open/create a name generator.
    InsufficientPermissions,
    /// Mismatching iceoryx2 versions.
    VersionMismatch,
    /// An internal error has occurred.
    InternalError,
}

impl core::fmt::Display for NameGeneratorOpenOrCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NameGeneratorOpenOrCreateError::{self:?}")
    }
}

impl core::error::Error for NameGeneratorOpenOrCreateError {}

/// Generates [`NodeName`]s and [`PortName`]s.
pub trait NameGenerator: Debug + Sized {
    /// Opens an existing [`NameGenerator`] or creates it.
    fn open_or_create(config: &Config) -> Result<Self, NameGeneratorOpenOrCreateError>;

    /// Generates a [`NodeName`].
    fn generate_node_name(&self) -> NodeName;

    /// Generates a [`PortName`].
    fn generate_port_name(&self) -> PortName;
}
