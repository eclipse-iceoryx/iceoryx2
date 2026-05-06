// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Error type for [`crate::service::Service`] operations.

use crate::connection;

/// Errors returned by [`crate::service::Service::open_or_create`],
/// [`crate::port_factory::DmabufPortFactory`] builder methods, and
/// [`crate::service_publisher::DmaBufServicePublisher`] /
/// [`crate::service_subscriber::DmaBufServiceSubscriber`] port operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum ServiceError {
    /// An iceoryx2 operation (node create, service open/create, or port build) failed.
    Iceoryx(String),
    /// The fd-channel (Unix-domain socket) reported an error.
    Connection(connection::Error),
    /// A filesystem I/O operation failed (e.g. creating the socket directory).
    Io(std::io::Error),
}

impl core::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Iceoryx(msg) => write!(f, "iceoryx2 error: {msg}"),
            Self::Connection(e) => write!(f, "fd-channel error: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl core::error::Error for ServiceError {}

impl From<connection::Error> for ServiceError {
    fn from(e: connection::Error) -> Self {
        Self::Connection(e)
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
