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

//! `SCM_RIGHTS` Unix-domain socket side-channel (Linux only).
//!
//! # Wire format
//!
//! Each message is:
//! ```text
//! [ 8 bytes: correlation token (u64 little-endian) ][ ancillary: 1 × SCM_RIGHTS fd ]
//! ```
//!
//! The publisher sends this once per connected subscriber via one `sendmsg(2)`
//! call.  The subscriber loops with `poll(2)` + `recvmsg(2)` until the token
//! matches the expected value.
//!
//! # Dependency choice
//!
//! `rustix::net::sendmsg` / `rustix::net::recvmsg` are used for the ancillary
//! message layer (zero-cost, safe wrappers).  `libc` is used directly for
//! `SO_PEERCRED` via `getsockopt` because rustix 1.x does not re-export that
//! constant on all targets.
//!
//! On non-Linux targets every method immediately returns
//! [`FdSidecarError::UnsupportedPlatform`].

use crate::error::FdSidecarError;

// ── Non-Linux stub ────────────────────────────────────────────────────────────

#[cfg(not(target_os = "linux"))]
mod imp {
    use super::FdSidecarError;
    use iceoryx2::port::side_channel::Role;

    /// Stub publisher — non-Linux; every method returns `UnsupportedPlatform`.
    #[derive(Debug)]
    pub struct ScmRightsPublisher;

    /// Stub subscriber — non-Linux; every method returns `UnsupportedPlatform`.
    #[derive(Debug)]
    pub struct ScmRightsSubscriber;

    impl ScmRightsPublisher {
        /// Open a side-channel publisher for `service_name`.
        ///
        /// Always returns [`FdSidecarError::UnsupportedPlatform`] on non-Linux
        /// targets.
        pub fn open(_service_name: &str, _role: Role) -> Result<Self, FdSidecarError> {
            Err(FdSidecarError::UnsupportedPlatform)
        }

        /// Open by service name string (alias for `open`).
        pub fn new(_service_name: &str) -> Result<Self, FdSidecarError> {
            Err(FdSidecarError::UnsupportedPlatform)
        }

        /// Send `fd` annotated with `token` to all connected subscribers.
        ///
        /// Always returns [`FdSidecarError::UnsupportedPlatform`] on non-Linux
        /// targets.
        pub fn send_fd_impl(
            &self,
            _token: core::num::NonZeroU64,
            _fd: std::os::fd::BorrowedFd<'_>,
        ) -> Result<(), FdSidecarError> {
            Err(FdSidecarError::UnsupportedPlatform)
        }
    }

    impl iceoryx2::port::side_channel::SideChannel for ScmRightsPublisher {
        type Error = FdSidecarError;
        type Transport = ();
        fn open(
            _service_name: &iceoryx2::service::service_name::ServiceName,
            _role: Role,
        ) -> Result<Self, Self::Error> {
            Err(FdSidecarError::UnsupportedPlatform)
        }
        fn transport(&mut self) -> &mut Self::Transport {
            // SAFETY-NOTE: This branch is statically unreachable because
            // `ScmRightsPublisher::open` / `::new` always return
            // `Err(FdSidecarError::UnsupportedPlatform)` on non-Linux targets,
            // so no instance of this type can ever be constructed.
            unreachable!(
                "non-Linux stub: value cannot be constructed because open() always returns Err(Unsupported)"
            )
        }
    }

    impl crate::side_channel::FdSideChannel for ScmRightsPublisher {
        fn send_fd(
            &mut self,
            _token: core::num::NonZeroU64,
            _fd: std::os::fd::BorrowedFd<'_>,
        ) -> Result<(), FdSidecarError> {
            Err(FdSidecarError::UnsupportedPlatform)
        }
        fn recv_fd_matching(
            &mut self,
            _expected: core::num::NonZeroU64,
            _timeout: std::time::Duration,
        ) -> Result<std::os::fd::OwnedFd, FdSidecarError> {
            Err(FdSidecarError::SideChannelIo(std::io::Error::other(
                "publisher does not receive fds",
            )))
        }
    }

    impl ScmRightsSubscriber {
        /// Open a side-channel subscriber for `service_name`.
        ///
        /// Always returns [`FdSidecarError::UnsupportedPlatform`] on non-Linux
        /// targets.
        pub fn open(_service_name: &str, _role: Role) -> Result<Self, FdSidecarError> {
            Err(FdSidecarError::UnsupportedPlatform)
        }

        /// Open by service name string (alias for `open`).
        pub fn new(_service_name: &str) -> Result<Self, FdSidecarError> {
            Err(FdSidecarError::UnsupportedPlatform)
        }

        /// Receive the fd whose token equals `expected`.
        ///
        /// Always returns [`FdSidecarError::UnsupportedPlatform`] on non-Linux
        /// targets.
        pub fn recv_fd_matching_impl(
            &mut self,
            _expected: core::num::NonZeroU64,
            _timeout: std::time::Duration,
        ) -> Result<std::os::fd::OwnedFd, FdSidecarError> {
            Err(FdSidecarError::UnsupportedPlatform)
        }
    }

    impl iceoryx2::port::side_channel::SideChannel for ScmRightsSubscriber {
        type Error = FdSidecarError;
        type Transport = ();
        fn open(
            _service_name: &iceoryx2::service::service_name::ServiceName,
            _role: Role,
        ) -> Result<Self, Self::Error> {
            Err(FdSidecarError::UnsupportedPlatform)
        }
        fn transport(&mut self) -> &mut Self::Transport {
            // SAFETY-NOTE: This branch is statically unreachable because
            // `ScmRightsSubscriber::open` / `::new` always return
            // `Err(FdSidecarError::UnsupportedPlatform)` on non-Linux targets,
            // so no instance of this type can ever be constructed.
            unreachable!(
                "non-Linux stub: value cannot be constructed because open() always returns Err(Unsupported)"
            )
        }
    }

    impl crate::side_channel::FdSideChannel for ScmRightsSubscriber {
        fn send_fd(
            &mut self,
            _token: core::num::NonZeroU64,
            _fd: std::os::fd::BorrowedFd<'_>,
        ) -> Result<(), FdSidecarError> {
            Err(FdSidecarError::SideChannelIo(std::io::Error::other(
                "subscriber does not send fds",
            )))
        }
        fn recv_fd_matching(
            &mut self,
            _expected: core::num::NonZeroU64,
            _timeout: std::time::Duration,
        ) -> Result<std::os::fd::OwnedFd, FdSidecarError> {
            Err(FdSidecarError::UnsupportedPlatform)
        }
    }
}

// ── Linux implementation ──────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod imp {
    use super::FdSidecarError;
    use crate::path::uds_path_for_service;
    use core::num::NonZeroU64;
    use iceoryx2::port::side_channel::Role;
    use iceoryx2_pal_concurrency_sync::atomic::{AtomicBool, Ordering};
    use rustix::io::IoSlice;
    use rustix::io::IoSliceMut;
    use rustix::net::{RecvAncillaryBuffer, RecvAncillaryMessage, SendAncillaryBuffer};
    use rustix::net::{RecvFlags, SendAncillaryMessage, SendFlags};
    use std::os::fd::{AsFd as _, AsRawFd as _, BorrowedFd, OwnedFd};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    // ── Publisher ─────────────────────────────────────────────────────────────

    /// SCM_RIGHTS publisher — binds a UDS server socket and accepts connections
    /// from subscribers.
    ///
    /// The socket file is removed when the publisher is dropped.
    #[derive(Debug)]
    pub struct ScmRightsPublisher {
        socket_path: String,
        /// Listener kept for `SideChannel::transport()` access.
        pub(crate) listener: UnixListener,
        /// Connected subscriber streams; protected by a Mutex so the accept
        /// thread can push new connections while the main thread calls
        /// `send_fd`.
        subscribers: Arc<Mutex<Vec<UnixStream>>>,
        shutdown: Arc<AtomicBool>,
        accept_thread: Option<thread::JoinHandle<()>>,
    }

    impl ScmRightsPublisher {
        /// Open a side-channel publisher for `service_name`.
        ///
        /// Binds a Unix-domain socket at the path derived from `service_name`
        /// (see [`crate::path::uds_path_for_service`]).  Spawns a background
        /// thread that `accept()`s incoming subscriber connections.
        pub fn open(service_name: &str, _role: Role) -> Result<Self, FdSidecarError> {
            Self::new(service_name)
        }

        /// Open by service name string.
        pub fn new(service_name: &str) -> Result<Self, FdSidecarError> {
            let socket_path = uds_path_for_service(service_name);

            // Create base directory if needed.
            let base = std::path::Path::new(&socket_path).parent().ok_or_else(|| {
                FdSidecarError::SideChannelIo(std::io::Error::other(
                    "socket path has no parent directory",
                ))
            })?;
            std::fs::create_dir_all(base).map_err(FdSidecarError::SideChannelIo)?;

            // Remove stale socket file if present.
            let _ = std::fs::remove_file(&socket_path);

            let listener =
                UnixListener::bind(&socket_path).map_err(FdSidecarError::SideChannelIo)?;

            let subscribers: Arc<Mutex<Vec<UnixStream>>> = Arc::new(Mutex::new(Vec::new()));
            let shutdown = Arc::new(AtomicBool::new(false));

            let shutdown_clone = Arc::clone(&shutdown);
            let subs_clone = Arc::clone(&subscribers);
            let listener_clone = listener
                .try_clone()
                .map_err(FdSidecarError::SideChannelIo)?;

            let accept_thread = thread::spawn(move || {
                listener_clone.set_nonblocking(true).ok();
                while !shutdown_clone.load(Ordering::Relaxed) {
                    match listener_clone.accept() {
                        Ok((stream, _addr)) => {
                            #[cfg(feature = "peercred")]
                            {
                                if let Err(e) = check_peer_uid(&stream) {
                                    tracing::warn!(
                                        "peercred check failed, rejecting connection: {e}"
                                    );
                                    continue;
                                }
                            }
                            // Accepted — push into subscriber list.
                            if let Ok(mut subs) = subs_clone.lock() {
                                subs.push(stream);
                            }
                        }
                        Err(ref e)
                            if e.kind() == std::io::ErrorKind::WouldBlock
                                || e.kind() == std::io::ErrorKind::Interrupted =>
                        {
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(ref e) => {
                            tracing::error!(
                                target: "iceoryx2_dmabuf::scm",
                                error = %e,
                                "sidecar accept loop terminated on I/O error"
                            );
                            break;
                        }
                    }
                }
                // Discard listener binding.
                drop(listener_clone);
            });

            Ok(Self {
                socket_path,
                listener,
                subscribers,
                shutdown,
                accept_thread: Some(accept_thread),
            })
        }

        /// Send `fd` annotated with `token` to every connected subscriber.
        ///
        /// Wire format per message:
        /// ```text
        /// [ 8 bytes: token (u64 LE) ][ ancillary: 1 × SCM_RIGHTS fd ]
        /// ```
        ///
        /// Dead subscribers (broken pipe) are pruned from the list.
        ///
        /// On `EAGAIN`/`EWOULDBLOCK`, returns
        /// `FdSidecarError::SideChannelIo(WouldBlock)`.
        pub fn send_fd_impl(
            &self,
            token: NonZeroU64,
            fd: BorrowedFd<'_>,
        ) -> Result<(), FdSidecarError> {
            let token_bytes = token.get().to_le_bytes();
            let iov = [IoSlice::new(&token_bytes)];

            // Ancillary buffer for one fd.
            let mut space = [core::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
            let mut cmsg_buf = SendAncillaryBuffer::new(&mut space);
            let ok = cmsg_buf.push(SendAncillaryMessage::ScmRights(std::slice::from_ref(&fd)));
            if !ok {
                return Err(FdSidecarError::SideChannelIo(std::io::Error::other(
                    "ancillary buffer too small",
                )));
            }

            let mut subs = self.subscribers.lock().map_err(|_| {
                FdSidecarError::SideChannelIo(std::io::Error::other("lock poisoned"))
            })?;

            subs.retain(|stream| {
                // Re-create the ancillary buffer for each subscriber (it is
                // consumed by sendmsg).
                let mut space2 =
                    [core::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
                let mut cmsg2 = SendAncillaryBuffer::new(&mut space2);
                cmsg2.push(SendAncillaryMessage::ScmRights(std::slice::from_ref(&fd)));

                rustix::net::sendmsg(stream.as_fd(), &iov, &mut cmsg2, SendFlags::empty()).is_ok()
            });

            Ok(())
        }

        /// Inject a raw token+fd to all connected subscriber streams.
        ///
        /// Bypasses the normal NonZeroU64 check so tests can forge arbitrary
        /// tokens (e.g. 9999) to exercise TokenMismatch / NoFdInMessage paths.
        ///
        /// Enabled by the `test-utils` feature — not part of the stable public interface.
        #[cfg(feature = "test-utils")]
        pub fn inject_raw_for_test(
            &self,
            token: u64,
            fd: BorrowedFd<'_>,
        ) -> Result<(), FdSidecarError> {
            let token_bytes = token.to_le_bytes();
            let iov = [IoSlice::new(&token_bytes)];
            let mut subs = self.subscribers.lock().map_err(|_| {
                FdSidecarError::SideChannelIo(std::io::Error::other("lock poisoned"))
            })?;
            subs.retain(|stream| {
                let mut space2 =
                    [core::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
                let mut cmsg2 = SendAncillaryBuffer::new(&mut space2);
                cmsg2.push(SendAncillaryMessage::ScmRights(std::slice::from_ref(&fd)));
                rustix::net::sendmsg(stream.as_fd(), &iov, &mut cmsg2, SendFlags::empty()).is_ok()
            });
            Ok(())
        }
    }

    impl iceoryx2::port::side_channel::SideChannel for ScmRightsPublisher {
        type Error = FdSidecarError;
        type Transport = UnixListener;
        fn open(
            service_name: &iceoryx2::service::service_name::ServiceName,
            _role: Role,
        ) -> Result<Self, Self::Error> {
            ScmRightsPublisher::new(service_name.as_str())
        }
        fn transport(&mut self) -> &mut Self::Transport {
            &mut self.listener
        }
    }

    impl crate::side_channel::FdSideChannel for ScmRightsPublisher {
        fn send_fd(&mut self, token: NonZeroU64, fd: BorrowedFd<'_>) -> Result<(), FdSidecarError> {
            self.send_fd_impl(token, fd)
        }
        fn recv_fd_matching(
            &mut self,
            _expected: NonZeroU64,
            _timeout: Duration,
        ) -> Result<OwnedFd, FdSidecarError> {
            Err(FdSidecarError::SideChannelIo(std::io::Error::other(
                "publisher does not receive fds",
            )))
        }
    }

    impl Drop for ScmRightsPublisher {
        fn drop(&mut self) {
            self.shutdown.store(true, Ordering::Relaxed);
            let _ = std::fs::remove_file(&self.socket_path);
            if let Some(handle) = self.accept_thread.take() {
                let _ = handle.join();
            }
        }
    }

    // ── Subscriber ────────────────────────────────────────────────────────────

    /// SCM_RIGHTS subscriber — connects to the publisher's UDS socket and
    /// receives file descriptors via `SCM_RIGHTS` ancillary data.
    #[derive(Debug)]
    pub struct ScmRightsSubscriber {
        /// Connected stream to the publisher's socket.
        pub(crate) stream: UnixStream,
    }

    impl ScmRightsSubscriber {
        /// Open a side-channel subscriber for `service_name`.
        ///
        /// Connects to the publisher's UDS socket (which must already be
        /// listening).
        pub fn open(service_name: &str, _role: Role) -> Result<Self, FdSidecarError> {
            Self::new(service_name)
        }

        /// Open by service name string.
        pub fn new(service_name: &str) -> Result<Self, FdSidecarError> {
            let socket_path = uds_path_for_service(service_name);
            let stream =
                UnixStream::connect(&socket_path).map_err(FdSidecarError::SideChannelIo)?;
            // Non-blocking so that recv_fd_matching can poll.
            stream
                .set_nonblocking(true)
                .map_err(FdSidecarError::SideChannelIo)?;
            Ok(Self { stream })
        }

        /// Receive the fd whose correlation token equals `expected`.
        ///
        /// Loops with `poll(2)` + `recvmsg(2)`:
        /// - Token `< expected`: stale fd — drop it and continue.
        /// - Token `== expected`: return the fd.
        /// - Token `> expected`: out-of-order delivery —
        ///   [`FdSidecarError::TokenMismatch`].
        /// - Timeout elapsed: [`FdSidecarError::NoFdInMessage`].
        #[allow(unsafe_code)]
        pub fn recv_fd_matching_impl(
            &mut self,
            expected: NonZeroU64,
            timeout: Duration,
        ) -> Result<OwnedFd, FdSidecarError> {
            let deadline = std::time::Instant::now() + timeout;

            loop {
                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    return Err(FdSidecarError::NoFdInMessage);
                }

                // poll(2) to wait up to `remaining` for data.
                let timeout_ms = remaining.as_millis().try_into().unwrap_or(i32::MAX);
                // SAFETY: poll is a plain syscall; pfd is stack-allocated and
                // valid for the duration of the call.
                let ready = unsafe {
                    let mut pfd = libc::pollfd {
                        fd: self.stream.as_fd().as_raw_fd(),
                        events: libc::POLLIN,
                        revents: 0,
                    };
                    libc::poll(&raw mut pfd, 1, timeout_ms)
                };

                if ready == 0 {
                    // Timeout.
                    return Err(FdSidecarError::NoFdInMessage);
                }
                if ready < 0 {
                    let e = std::io::Error::last_os_error();
                    if e.kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    }
                    return Err(FdSidecarError::SideChannelIo(e));
                }

                // Receive the 8-byte token and the ancillary fd.
                let mut token_buf = [0u8; 8];
                let mut iov = [IoSliceMut::new(&mut token_buf)];
                let mut space =
                    [core::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
                let mut cmsg_buf = RecvAncillaryBuffer::new(&mut space);

                let result = rustix::net::recvmsg(
                    self.stream.as_fd(),
                    &mut iov,
                    &mut cmsg_buf,
                    RecvFlags::empty(),
                );

                match result {
                    Err(e)
                        if e == rustix::io::Errno::AGAIN || e == rustix::io::Errno::WOULDBLOCK =>
                    {
                        continue;
                    }
                    Err(e) if e == rustix::io::Errno::INTR => {
                        continue;
                    }
                    Err(e) => {
                        return Err(FdSidecarError::SideChannelIo(
                            std::io::Error::from_raw_os_error(e.raw_os_error()),
                        ));
                    }
                    Ok(msg) => {
                        if msg.bytes < 8 {
                            // Truncated — skip.
                            continue;
                        }
                    }
                }

                let got_token = u64::from_le_bytes(token_buf);
                let expected_raw = expected.get();

                // Extract the fd from ancillary data.
                let owned_fd = cmsg_buf
                    .drain()
                    .filter_map(|msg| {
                        if let RecvAncillaryMessage::ScmRights(mut it) = msg {
                            it.next()
                        } else {
                            None
                        }
                    })
                    .next();

                if got_token < expected_raw {
                    // Stale: drop the fd and continue waiting.
                    drop(owned_fd);
                    continue;
                }

                if got_token == expected_raw {
                    return owned_fd.ok_or(FdSidecarError::NoFdInMessage);
                }

                // got_token > expected_raw — out-of-order.
                drop(owned_fd);
                return Err(FdSidecarError::TokenMismatch {
                    expected: expected_raw,
                    got: got_token,
                });
            }
        }
    }

    impl iceoryx2::port::side_channel::SideChannel for ScmRightsSubscriber {
        type Error = FdSidecarError;
        type Transport = UnixStream;
        fn open(
            service_name: &iceoryx2::service::service_name::ServiceName,
            _role: Role,
        ) -> Result<Self, Self::Error> {
            ScmRightsSubscriber::new(service_name.as_str())
        }
        fn transport(&mut self) -> &mut Self::Transport {
            &mut self.stream
        }
    }

    impl crate::side_channel::FdSideChannel for ScmRightsSubscriber {
        fn send_fd(
            &mut self,
            _token: NonZeroU64,
            _fd: BorrowedFd<'_>,
        ) -> Result<(), FdSidecarError> {
            Err(FdSidecarError::SideChannelIo(std::io::Error::other(
                "subscriber does not send fds",
            )))
        }
        fn recv_fd_matching(
            &mut self,
            expected: NonZeroU64,
            timeout: Duration,
        ) -> Result<OwnedFd, FdSidecarError> {
            self.recv_fd_matching_impl(expected, timeout)
        }
    }

    impl Drop for ScmRightsSubscriber {
        fn drop(&mut self) {
            // Stream is closed when dropped — publisher prunes dead subscribers
            // on the next send_fd call (broken-pipe pruning).
        }
    }

    // ── peercred check ────────────────────────────────────────────────────────

    /// Validate that the peer connected to `stream` has the same effective UID
    /// as the current process (Linux `SO_PEERCRED`).
    ///
    /// Returns `Ok(())` if the UIDs match; `Err(FdSidecarError::PeerUidMismatch)`
    /// otherwise.
    #[cfg(feature = "peercred")]
    #[allow(unsafe_code)]
    pub(crate) fn check_peer_uid(stream: &UnixStream) -> crate::error::Result<()> {
        use std::os::fd::AsRawFd as _;

        // SAFETY: getsockopt with SO_PEERCRED is a valid operation on a connected
        // Unix-domain socket fd; ucred is a plain C struct with no ownership.
        let cred: libc::ucred = unsafe {
            let mut val: libc::ucred = std::mem::zeroed();
            let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
            let rc = libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                &raw mut val as *mut libc::c_void,
                &raw mut len,
            );
            if rc != 0 {
                return Err(crate::error::FdSidecarError::SideChannelIo(
                    std::io::Error::last_os_error(),
                ));
            }
            val
        };

        // SAFETY: geteuid() is always safe.
        let expected_uid = unsafe { libc::geteuid() };
        if cred.uid != expected_uid {
            return Err(crate::error::FdSidecarError::PeerUidMismatch {
                peer_uid: cred.uid,
                expected_uid,
            });
        }
        Ok(())
    }
}

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use imp::{ScmRightsPublisher, ScmRightsSubscriber};
