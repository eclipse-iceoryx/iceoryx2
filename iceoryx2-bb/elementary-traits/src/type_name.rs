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

/// Trait for types that can be uniquely identified by their [`TypeName::type_name()`].
pub unsafe trait TypeName {
    /// The unique identifier of the type. It shall be used to identify a specific type across
    /// processes and languages.
    ///
    /// # Safety
    ///
    ///  * The user must guarantee that all types, also definitions in different languages, have
    ///    the same memory layout.
    unsafe fn type_name() -> &'static str {
        core::any::type_name::<Self>()
    }
}
