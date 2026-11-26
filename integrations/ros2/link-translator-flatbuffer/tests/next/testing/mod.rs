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

pub mod messages;

use iceoryx2_integrations_ros2_link_adapter::typesupport::{self, IntrospectionKind};
use iceoryx2_integrations_ros2_link_translator_flatbuffer::next::{
    self as translator, CompileError, Compiled,
};
use iceoryx2_link_backend::service_description::Schema;

pub fn compile(schema: &[u8], message: &str) -> Result<Compiled, CompileError> {
    let schema = Schema::new(schema.to_vec()).expect("is a schema");
    let typesupport = typesupport::get::<IntrospectionKind>(message).expect("is loaded");
    translator::compile(&schema, &typesupport)
}
