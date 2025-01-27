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

/// A type implementing Identifiable can be uniquely identified by its [`Identifiable::type_name()`]
/// in an inter-process communication context.
///
/// # Safety
///
///  * The user must guarantee that all types, also definitions in different languages, have
///    the same memory layout.
///
pub unsafe trait Identifiable {
    /// The unique identifier of the type. It shall be used to identify a specific type accross
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
