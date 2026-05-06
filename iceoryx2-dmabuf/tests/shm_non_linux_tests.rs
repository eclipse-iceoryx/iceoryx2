// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

// TEST 2 — spec §12: non-Linux stub always returns Err(Unsupported).
//
// NOTE: shm::non_linux is pub(crate), so NonLinux cannot be imported from
// integration test files. The authoritative unit test for the shm NonLinux
// stub lives in src/shm/non_linux.rs as an inline #[cfg(test)] mod.
//
// This integration test file exercises the public connection::non_linux::NonLinux
// stub (which IS public) to validate the non-Linux platform-error path, and
// keeps the [[test]] binary non-empty on both platforms.

// On Linux: trivial smoke — confirms the binary compiles.
#[cfg(target_os = "linux")]
#[test]
fn shm_non_linux_stub_existence_verified_by_build() {
    // shm::non_linux is pub(crate). The authoritative test is the inline
    // mod tests in src/shm/non_linux.rs (run as a lib unit test).
    // On Linux, this test is a no-op placeholder.
}

// On non-Linux: exercise the public connection NonLinux stub.
#[cfg(not(target_os = "linux"))]
#[test]
fn non_linux_connection_stub_returns_unsupported() {
    use iceoryx2_dmabuf::connection::{Error, FdPassingConnection, NonLinux};

    let stub = NonLinux;

    // recv_with_fd takes no fd argument — safe to call unconditionally.
    let result = stub.recv_with_fd();
    assert!(result.is_err(), "NonLinux recv_with_fd must return Err");
    if let Err(e) = result {
        assert!(
            matches!(e, Error::UnsupportedPlatform),
            "expected UnsupportedPlatform, got {e}"
        );
    }

    // recv_release_ack also takes no fd.
    let ack_result = stub.recv_release_ack();
    assert!(
        ack_result.is_err(),
        "NonLinux recv_release_ack must return Err"
    );
    if let Err(e) = ack_result {
        assert!(
            matches!(e, Error::UnsupportedPlatform),
            "expected UnsupportedPlatform, got {e}"
        );
    }
}
