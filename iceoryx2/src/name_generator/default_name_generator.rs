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

//! Generates default [`NodeName`]s and [`PortName`]s.

pub use crate::name_generator::*;

/// Generator for default [`NodeName`]s and [`PortName`]s.
#[derive(Debug)]
pub struct DefaultNameGenerator {}

impl NameGenerator for DefaultNameGenerator {
    fn open_or_create(_: &Config) -> Result<Self, ()> {
        Ok(Self {})
    }

    fn generate_node_name(&self) -> NodeName {
        NodeName::default()
    }

    fn generate_port_name(&self) -> PortName {
        PortName::default()
    }
}
