// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! UDS socket path derivation for the iceoryx2-dmabuf side-channel.
//!
//! Each service maps to a deterministic path under a configurable base
//! directory.  The filename is a 40-char lower-hex SHA-1 digest of the
//! service name, suffixed with `.sock` (45 chars total).
//!
//! The base directory defaults to `/tmp/iox2-dmabuf/` but can be overridden
//! at test time via the `ICEORYX2_DMABUF_SOCKET_DIR` environment variable.

use sha1_smol::Sha1;

/// Default base directory for side-channel sockets.
const DEFAULT_SOCKET_DIR: &str = "/tmp/iox2-dmabuf";

/// Derive the Unix-domain socket path for a given `service_name`.
///
/// The path is deterministic: the same name always yields the same path.
/// Different names are collision-resistant (SHA-1, 160-bit output space).
///
/// Base directory: `/tmp/iox2-dmabuf/` (override with
/// `ICEORYX2_DMABUF_SOCKET_DIR` env var — useful in tests to isolate
/// concurrent test runs).
///
/// The returned path has the form `<base>/<40-hex-sha1>.sock`.
pub fn uds_path_for_service(service_name: &str) -> String {
    let base = match std::env::var("ICEORYX2_DMABUF_SOCKET_DIR") {
        Ok(v) => v,
        Err(_) => DEFAULT_SOCKET_DIR.to_owned(),
    };
    let mut h = Sha1::new();
    h.update(service_name.as_bytes());
    format!("{}/{}.sock", base, h.digest())
}
