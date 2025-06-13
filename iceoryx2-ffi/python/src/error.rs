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

use pyo3::create_exception;
use pyo3::exceptions::PyException;

create_exception!(
    iceoryx2_ffi_python,
    SemanticStringError,
    PyException,
    "Errors caused by creating a semantic string."
);

create_exception!(
    iceoryx2_ffi_python,
    ConfigCreationError,
    PyException,
    "Errors caused by creating a new config."
);

create_exception!(
    iceoryx2_ffi_python,
    NodeCreationFailure,
    PyException,
    "Errors caused by creating a new node."
);

create_exception!(
    iceoryx2_ffi_python,
    NodeWaitFailure,
    PyException,
    "Errors caused by creating a new node."
);
