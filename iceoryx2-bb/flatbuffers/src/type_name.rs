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

#[derive(Debug, PartialEq, Eq)]
pub struct TypeName {
    pub name: &'static str,
    pub namespace: &'static str,
}

pub fn type_name<T>() -> TypeName {
    let full_name = core::any::type_name::<T>();
    let mut namespace = "";
    let mut name = full_name;

    if let Some(pos) = full_name.rfind("::") {
        namespace = &full_name[..pos];
        name = &full_name[pos + 2..];
    }

    if let Some(pos) = name.rfind("<") {
        name = &name[..pos];
    }

    if let Some(pos) = namespace.rfind("::") {
        namespace = &namespace[pos + 2..];
    }

    TypeName { name, namespace }
}
