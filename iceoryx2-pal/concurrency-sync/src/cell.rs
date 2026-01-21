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

#[allow(clippy::disallowed_types)]
pub type Cell<T> = core::cell::Cell<T>;

#[allow(clippy::disallowed_types)]
pub type OnceCell<T> = core::cell::OnceCell<T>;

#[allow(clippy::disallowed_types)]
pub type Ref<'a, T> = core::cell::Ref<'a, T>;

#[allow(clippy::disallowed_types)]
pub type RefCell<T> = core::cell::RefCell<T>;

#[allow(clippy::disallowed_types)]
pub type RefMut<'a, T> = core::cell::RefMut<'a, T>;

#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type UnsafeCell<T> = core::cell::UnsafeCell<T>;

#[cfg(all(test, loom, feature = "std"))]
pub(crate) type UnsafeCell<T> = loom::cell::UnsafeCell<T>;
