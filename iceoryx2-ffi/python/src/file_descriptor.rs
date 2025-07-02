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

use std::sync::Arc;

use pyo3::prelude::*;

#[derive(Debug)]
#[pyclass(str = "{0:?}")]
/// Represents a FileDescriptor in a POSIX system. Contains always a value greater or equal zero,
/// a valid file descriptor. It takes the ownership of the provided file descriptor and calls
/// `posix::close` on destruction.
pub struct FileDescriptor(pub(crate) Arc<iceoryx2::prelude::FileDescriptor>);

impl iceoryx2::prelude::FileDescriptorBased for FileDescriptor {
    fn file_descriptor(&self) -> &iceoryx2::prelude::FileDescriptor {
        self.0.file_descriptor()
    }
}

impl iceoryx2::prelude::SynchronousMultiplexing for FileDescriptor {}

#[pymethods]
impl FileDescriptor {
    #[staticmethod]
    /// Creates a FileDescriptor which does not hold the ownership of the file descriptor and will
    /// not call `posix::close` on destruction.
    pub fn non_owning_new(value: i32) -> Option<FileDescriptor> {
        iceoryx2::prelude::FileDescriptor::non_owning_new(value).map(|v| Self(Arc::new(v)))
    }

    #[staticmethod]
    /// Creates a new FileDescriptor. If the value is smaller than zero it returns [`None`].
    pub fn new(value: i32) -> Option<FileDescriptor> {
        iceoryx2::prelude::FileDescriptor::new(value).map(|v| Self(Arc::new(v)))
    }

    #[getter]
    /// Returns the underlying value of the FileDescriptor
    ///
    /// # Safety
    ///
    ///  * the user shall not store the value in a variable otherwise lifetime issues may be
    ///    encountered
    ///  * do not manually close the file descriptor with a sys call
    ///
    pub fn native_handle(&self) -> i32 {
        unsafe { self.0.native_handle() }
    }
}
