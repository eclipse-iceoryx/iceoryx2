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

use pyo3::prelude::*;

#[pyclass]
#[derive(PartialEq, Clone, Debug)]
/// Defines if the type is a slice with a runtime-size (`TypeVariant::Dynamic`)
/// or if its a type that satisfies `Sized` (`TypeVariant::FixedSize`).
pub enum TypeVariant {
    /// A fixed size type like `uint64_t`
    FixedSize,
    /// A dynamic sized type like a slice (dynamic array)
    Dynamic,
}

impl From<iceoryx2::service::static_config::message_type_details::TypeVariant> for TypeVariant {
    fn from(
        value: iceoryx2::service::static_config::message_type_details::TypeVariant,
    ) -> TypeVariant {
        match value {
            iceoryx2::service::static_config::message_type_details::TypeVariant::Dynamic => {
                TypeVariant::Dynamic
            }
            iceoryx2::service::static_config::message_type_details::TypeVariant::FixedSize => {
                TypeVariant::FixedSize
            }
        }
    }
}

impl From<TypeVariant> for iceoryx2::service::static_config::message_type_details::TypeVariant {
    fn from(
        value: TypeVariant,
    ) -> iceoryx2::service::static_config::message_type_details::TypeVariant {
        match value {
            TypeVariant::Dynamic => {
                iceoryx2::service::static_config::message_type_details::TypeVariant::Dynamic
            }
            TypeVariant::FixedSize => {
                iceoryx2::service::static_config::message_type_details::TypeVariant::FixedSize
            }
        }
    }
}
