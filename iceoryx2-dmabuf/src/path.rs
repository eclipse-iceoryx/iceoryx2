// SPDX-License-Identifier: Apache-2.0 OR MIT

//! UDS socket path derivation for the `dmabuf::Service` fd channel.
//!
//! Each service maps to a deterministic path under a configurable base
//! directory. The filename is a 16-char lower-hex hash of the service name
//! (from [`std::collections::hash_map::DefaultHasher`]), suffixed with `.sock`.
//!
//! The base directory defaults to `/tmp/iox2-dmabuf/` but can be overridden
//! at test time via the `ICEORYX2_DMABUF_SOCKET_DIR` environment variable.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _, Hasher as _};

/// Default base directory for fd-channel sockets.
const DEFAULT_SOCKET_DIR: &str = "/tmp/iox2-dmabuf";

/// Derive the Unix-domain socket path for a given `service_name`.
///
/// The path is deterministic: the same name always yields the same path on the
/// same platform (note: `DefaultHasher` is not guaranteed stable across Rust
/// releases, but is stable within a single compilation unit and sufficient for
/// process-local socket coordination).
///
/// Base directory: `/tmp/iox2-dmabuf/` (override with
/// `ICEORYX2_DMABUF_SOCKET_DIR` env var — useful in tests to isolate
/// concurrent test runs).
///
/// The returned path has the form `<base>/<16-hex-u64>.sock`.
pub fn uds_path_for_service(service_name: &str) -> String {
    let base = std::env::var("ICEORYX2_DMABUF_SOCKET_DIR")
        .unwrap_or_else(|_| DEFAULT_SOCKET_DIR.to_owned());
    let mut h = DefaultHasher::new();
    service_name.hash(&mut h);
    let digest = h.finish();
    format!("{base}/{digest:016x}.sock")
}
