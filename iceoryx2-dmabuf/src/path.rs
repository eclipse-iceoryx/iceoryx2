// SPDX-License-Identifier: Apache-2.0 OR MIT

//! UDS socket path derivation for the `dmabuf::Service` fd channel.
//!
//! Each service maps to a deterministic path under a configurable base
//! directory. The filename is a 16-char lower-hex FNV-1a 64-bit hash of the
//! service name, suffixed with `.sock`.
//!
//! The base directory defaults to `/tmp/iox2-dmabuf/` but can be overridden
//! at test time via the `ICEORYX2_DMABUF_SOCKET_DIR` environment variable.

/// Default base directory for fd-channel sockets.
const DEFAULT_SOCKET_DIR: &str = "/tmp/iox2-dmabuf";

/// FNV-1a 64-bit hash — deterministic across all Rust versions and platforms.
///
/// Using `DefaultHasher` would produce different paths across rustc releases,
/// silently breaking service discovery between binaries built with different
/// toolchains. FNV-1a is lock-in-free (no dep), O(n), and collision-resistant
/// enough for socket-file naming.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xCBF2_9CE4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01B3;
    let mut h = OFFSET_BASIS;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(PRIME);
    }
    h
}

/// Derive the Unix-domain socket path for a given `service_name`.
///
/// The path is fully deterministic across Rust versions (uses inline FNV-1a 64,
/// not `DefaultHasher`).
///
/// Base directory: `/tmp/iox2-dmabuf/` (override with
/// `ICEORYX2_DMABUF_SOCKET_DIR` env var — useful in tests to isolate
/// concurrent test runs).
///
/// The returned path has the form `<base>/<16-hex-u64>.sock`.
pub fn uds_path_for_service(service_name: &str) -> String {
    let base = std::env::var("ICEORYX2_DMABUF_SOCKET_DIR")
        .unwrap_or_else(|_| DEFAULT_SOCKET_DIR.to_owned());
    let digest = fnv1a_64(service_name.as_bytes());
    format!("{base}/{digest:016x}.sock")
}

// TEST 9 — unit tests for path derivation.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_path_for_same_service_name() {
        let a = uds_path_for_service("dmabuf/test");
        let b = uds_path_for_service("dmabuf/test");
        assert_eq!(a, b);
    }

    #[test]
    fn distinct_path_for_different_service_names() {
        let a = uds_path_for_service("svc/a");
        let b = uds_path_for_service("svc/b");
        assert_ne!(a, b);
    }

    #[test]
    fn path_format_contains_iox2_dmabuf_and_ends_with_sock() {
        let p = uds_path_for_service("test");
        assert!(p.contains("iox2-dmabuf"), "got {p}");
        assert!(p.ends_with(".sock"), "got {p}");
    }
}
