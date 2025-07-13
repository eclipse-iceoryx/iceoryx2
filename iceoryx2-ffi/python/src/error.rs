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
    ConfigCreationError,
    PyException,
    "Errors caused by creating a new config."
);

create_exception!(
    iceoryx2_ffi_python,
    ClientCreateError,
    PyException,
    "Errors caused when creating a new client port."
);

create_exception!(
    iceoryx2_ffi_python,
    ConnectionFailure,
    PyException,
    "Errors caused a connection to an endpoint could not be established."
);

create_exception!(
    iceoryx2_ffi_python,
    LoanError,
    PyException,
    "Errors caused when loaning memory from a ports datasegment."
);

create_exception!(
    iceoryx2_ffi_python,
    ListenerCreateError,
    PyException,
    "Errors caused when creating a new Listener port."
);

create_exception!(
    iceoryx2_ffi_python,
    NodeCreationFailure,
    PyException,
    "Errors caused by creating a new node."
);

create_exception!(
    iceoryx2_ffi_python,
    NodeCleanupFailure,
    PyException,
    "Errors caused by cleaning up the stale resources of a dead node."
);

create_exception!(
    iceoryx2_ffi_python,
    NodeListFailure,
    PyException,
    "Errors caused when listing all nodes."
);

create_exception!(
    iceoryx2_ffi_python,
    NodeWaitFailure,
    PyException,
    "Errors caused when waiting on a node."
);

create_exception!(
    iceoryx2_ffi_python,
    NotifierCreateError,
    PyException,
    "Errors caused when creating a new Notifier port."
);

create_exception!(
    iceoryx2_ffi_python,
    NotifierNotifyError,
    PyException,
    "Errors caused when sending a notification via the Notifier port in an event service."
);

create_exception!(
    iceoryx2_ffi_python,
    EventOpenError,
    PyException,
    "Errors caused when opening an event service."
);

create_exception!(
    iceoryx2_ffi_python,
    EventCreateError,
    PyException,
    "Errors caused when creating an event service."
);

create_exception!(
    iceoryx2_ffi_python,
    EventOpenOrCreateError,
    PyException,
    "Errors caused when open or creating an event service."
);

create_exception!(
    iceoryx2_ffi_python,
    InvalidAlignmentValue,
    PyException,
    "Errors caused when the value of the alignment is not a power of two or exceeds the maximum supported value."
);

create_exception!(
    iceoryx2_ffi_python,
    ListenerWaitError,
    PyException,
    "Errors caused when waiting on a Listener port in an event service."
);

create_exception!(
    iceoryx2_ffi_python,
    PublisherCreateError,
    PyException,
    "Errors caused when creating a publisher port."
);

create_exception!(
    iceoryx2_ffi_python,
    PublishSubscribeOpenError,
    PyException,
    "Errors caused when opening a publish-subscribe service."
);

create_exception!(
    iceoryx2_ffi_python,
    PublishSubscribeCreateError,
    PyException,
    "Errors caused when creating a publish-subscribe service."
);

create_exception!(
    iceoryx2_ffi_python,
    PublishSubscribeOpenOrCreateError,
    PyException,
    "Errors caused when open or creating a publish-subscribe service."
);

create_exception!(
    iceoryx2_ffi_python,
    ReceiveError,
    PyException,
    "Errors caused when receiving data."
);

create_exception!(
    iceoryx2_ffi_python,
    RequestResponseOpenError,
    PyException,
    "Errors caused when opening a request-response service."
);

create_exception!(
    iceoryx2_ffi_python,
    RequestResponseCreateError,
    PyException,
    "Errors caused when creating a request-response service."
);

create_exception!(
    iceoryx2_ffi_python,
    RequestResponseOpenOrCreateError,
    PyException,
    "Errors caused when open or creating a request-response service."
);

create_exception!(
    iceoryx2_ffi_python,
    RequestSendError,
    PyException,
    "Errors caused when sending a request."
);

create_exception!(
    iceoryx2_ffi_python,
    SemanticStringError,
    PyException,
    "Errors caused by creating a semantic string."
);

create_exception!(
    iceoryx2_ffi_python,
    SendError,
    PyException,
    "Errors caused when sending data."
);

create_exception!(
    iceoryx2_ffi_python,
    ServiceDetailsError,
    PyException,
    "Errors caused when acquiring the details of a service."
);

create_exception!(
    iceoryx2_ffi_python,
    ServiceListError,
    PyException,
    "Errors caused when listing all existing services."
);

create_exception!(
    iceoryx2_ffi_python,
    ServerCreateError,
    PyException,
    "Errors caused when creating a server port."
);

create_exception!(
    iceoryx2_ffi_python,
    SubscriberCreateError,
    PyException,
    "Errors caused when creating a subscriber port."
);

create_exception!(
    iceoryx2_ffi_python,
    WaitSetAttachmentError,
    PyException,
    "Errors caused when attaching something to the waitset."
);

create_exception!(
    iceoryx2_ffi_python,
    WaitSetCreateError,
    PyException,
    "Errors caused by creating a new waitset."
);

create_exception!(
    iceoryx2_ffi_python,
    WaitSetRunError,
    PyException,
    "Errors caused by calling WaitSet::wait_and_process()."
);
