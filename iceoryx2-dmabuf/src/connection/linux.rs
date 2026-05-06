// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Linux implementation of [`FdPassingConnection`] via `SCM_RIGHTS`.
//!
//! Two distinct struct types separate the roles per SRP:
//! - [`LinuxPublisher`]: binds a UDS socket, accept-loop thread, fanout send.
//! - [`LinuxSubscriber`]: connects to the socket, non-blocking poll/recvmsg.
//!
//! A zero-overhead namespace struct [`Linux`] provides:
//! - `Linux::open_publisher(path)` → `Result<LinuxPublisher>`
//! - `Linux::open_subscriber(path)` → `Result<LinuxSubscriber>`
//!
//! Wire format (v2) — matches `connection.rs` header:
//!
//! ## Forward frames (publisher → subscriber, fd-carrying):
//! ```text
//! [8B payload_len u64 LE][8B token u64 LE][SCM_RIGHTS ancillary: 1 fd]
//! ```
//!
//! ## Back-channel ack frames (subscriber → publisher, no ancillary):
//! ```text
//! [8B magic u64 LE (low-32 = 0x4D4F5346, high-32 = 0)][8B token u64 LE]
//! ```
//!
//! Disambiguation: ancillary present = forward fd frame; no ancillary = ack.

use super::{Error, FdPassingConnection, Result};
use rustix::net::{
    RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags, SendAncillaryBuffer,
    SendAncillaryMessage, SendFlags,
};
use std::io::{IoSlice, IoSliceMut, Write as _};
use std::os::fd::{AsFd as _, BorrowedFd, OwnedFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// ── Wire format constants (v2) ────────────────────────────────────────────────

/// Total iov payload size for both forward and ack frames (16 bytes).
const HDR_LEN: usize = 16;

/// Byte offset of payload_len in a forward frame (bytes 0..8).
const PAYLOAD_LEN_OFFSET: usize = 0;

/// Byte offset of token in a forward frame (bytes 8..16).
const TOKEN_OFFSET: usize = 8;

/// Byte offset of magic sentinel in an ack frame (bytes 0..8).
const ACK_MAGIC_OFFSET: usize = 0;

/// Byte offset of token in an ack frame (bytes 8..16).
const ACK_TOKEN_OFFSET: usize = 8;

/// Magic sentinel for ack frames: ASCII `b"MOSF"` as little-endian u32 in
/// the low 32 bits, zero in the high 32 bits.
const MAGIC_BUFFER_RELEASED: u32 = 0x4D4F_5346;

// ── LinuxPublisher ────────────────────────────────────────────────────────────

/// UDS publisher that binds a socket, accepts connections in a background
/// thread, and fans out each fd to all connected subscribers via `SCM_RIGHTS`.
///
/// The socket file is removed on [`Drop`].
pub struct LinuxPublisher {
    socket_path: String,
    subscribers: Arc<Mutex<Vec<UnixStream>>>,
    shutdown: Arc<AtomicBool>,
    accept_thread: Option<thread::JoinHandle<()>>,
}

impl LinuxPublisher {
    /// Bind `socket_path`, spawn the accept thread, and return a publisher.
    ///
    /// Any stale socket file at `socket_path` is removed before binding.
    pub fn open(socket_path: &str) -> Result<Self> {
        // Remove stale socket file if present.
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path)?;

        let subscribers: Arc<Mutex<Vec<UnixStream>>> = Arc::new(Mutex::new(Vec::new()));
        let shutdown = Arc::new(AtomicBool::new(false));

        let subs_clone = Arc::clone(&subscribers);
        let shutdown_clone = Arc::clone(&shutdown);
        let listener_clone = listener.try_clone()?;
        listener_clone
            .set_nonblocking(true)
            .map_err(|e| std::io::Error::new(e.kind(), format!("set_nonblocking: {e}")))?;

        let accept_thread = thread::spawn(move || {
            while !shutdown_clone.load(Ordering::Relaxed) {
                match listener_clone.accept() {
                    Ok((stream, _addr)) => {
                        #[cfg(feature = "peercred")]
                        {
                            if let Err(_e) = check_peer_uid(&stream) {
                                #[cfg(debug_assertions)]
                                eprintln!(
                                    "iceoryx2-dmabuf: peercred check failed, rejecting connection"
                                );
                                continue;
                            }
                        }
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
                    Err(_e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("iceoryx2-dmabuf: accept loop terminated: {_e}");
                        break;
                    }
                }
            }
            drop(listener_clone);
        });

        Ok(Self {
            socket_path: socket_path.to_owned(),
            subscribers,
            shutdown,
            accept_thread: Some(accept_thread),
        })
    }

    /// Number of currently connected subscribers.
    ///
    /// Intended for tests and diagnostics; not part of the hot path.
    ///
    /// # Errors
    ///
    /// Returns [`Error::LockPoisoned`] if the internal subscriber mutex was
    /// poisoned by a panicking thread.
    pub fn connected_subscriber_count(&self) -> super::Result<usize> {
        Ok(self
            .subscribers
            .lock()
            .map_err(|_| Error::LockPoisoned)?
            .len())
    }
}

impl FdPassingConnection for LinuxPublisher {
    fn send_with_fd(&self, fd: BorrowedFd<'_>, len: u64, token: u64) -> Result<()> {
        // Wire v2 forward header: [payload_len 8B LE][token 8B LE].
        let mut hdr = [0u8; HDR_LEN];
        hdr[PAYLOAD_LEN_OFFSET..TOKEN_OFFSET].copy_from_slice(&len.to_le_bytes());
        hdr[TOKEN_OFFSET..HDR_LEN].copy_from_slice(&token.to_le_bytes());
        let iov = [IoSlice::new(&hdr)];

        let mut subs = self.subscribers.lock().map_err(|_| Error::LockPoisoned)?;

        subs.retain(|stream| {
            // Re-create the ancillary buffer for each subscriber (consumed by sendmsg).
            let mut space = [core::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
            let mut cmsg = SendAncillaryBuffer::new(&mut space);
            cmsg.push(SendAncillaryMessage::ScmRights(core::slice::from_ref(&fd)));

            rustix::net::sendmsg(stream.as_fd(), &iov, &mut cmsg, SendFlags::empty()).is_ok()
        });

        Ok(())
    }

    fn recv_with_fd(&self) -> Result<Option<(OwnedFd, u64, u64)>> {
        // Publisher role never receives forward frames — always empty.
        Ok(None)
    }

    fn send_release_ack(&self, _token: u64) -> Result<()> {
        // Publisher role does not send acks — acks flow subscriber → publisher.
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "publisher cannot send release acks",
        )))
    }

    fn recv_release_ack(&self) -> Result<Option<u64>> {
        let mut subs = self.subscribers.lock().map_err(|_| Error::LockPoisoned)?;

        let mut dead_indices: Vec<usize> = Vec::new();
        let mut found_token: Option<u64> = None;

        'outer: for (i, stream) in subs.iter_mut().enumerate() {
            // Non-blocking poll: check if data is available on this subscriber stream.
            // SAFETY: poll(2) is a plain Linux syscall; pfd is stack-allocated and
            // valid for the duration of the call. We use libc directly because rustix
            // does not expose a zero-timeout poll shorthand in v1.x.
            #[allow(unsafe_code)]
            let ready = unsafe {
                let mut pfd = libc::pollfd {
                    fd: stream.as_fd().as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                libc::poll(core::ptr::addr_of_mut!(pfd), 1, 0)
            };

            if ready == 0 {
                continue; // No data on this subscriber — check next.
            }
            if ready < 0 {
                let e = std::io::Error::last_os_error();
                if e.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                // Propagate non-EINTR poll errors after pruning dead streams.
                // Remove dead streams first to avoid CPU spin on future calls.
                for idx in dead_indices.into_iter().rev() {
                    subs.remove(idx);
                }
                return Err(Error::Io(e));
            }

            // Data available — receive the 16-byte frame.
            let mut hdr = [0u8; HDR_LEN];
            let mut iov = [IoSliceMut::new(&mut hdr)];
            // Use a space large enough for SCM_RIGHTS(1) — so we can detect drift.
            let mut space = [core::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
            let mut cmsg_buf = RecvAncillaryBuffer::new(&mut space);

            let result =
                rustix::net::recvmsg(stream.as_fd(), &mut iov, &mut cmsg_buf, RecvFlags::empty());

            match result {
                Err(e) if e == rustix::io::Errno::AGAIN || e == rustix::io::Errno::WOULDBLOCK => {
                    continue;
                }
                Err(e) if e == rustix::io::Errno::INTR => {
                    continue;
                }
                Err(e) => {
                    for idx in dead_indices.into_iter().rev() {
                        subs.remove(idx);
                    }
                    return Err(Error::Io(std::io::Error::from_raw_os_error(
                        e.raw_os_error(),
                    )));
                }
                Ok(msg) => {
                    if msg.bytes == 0 {
                        // Peer disconnected — mark for pruning to avoid POLLHUP spin.
                        dead_indices.push(i);
                        continue;
                    }
                    if msg.bytes < HDR_LEN {
                        for idx in dead_indices.into_iter().rev() {
                            subs.remove(idx);
                        }
                        return Err(Error::Truncated {
                            got: msg.bytes,
                            want: HDR_LEN,
                        });
                    }
                }
            }

            // Disambiguation: if ancillary data is present, this is a forward fd
            // frame that arrived on the back-channel — protocol drift.
            let has_ancillary = cmsg_buf.drain().next().is_some();
            if has_ancillary {
                for idx in dead_indices.into_iter().rev() {
                    subs.remove(idx);
                }
                return Err(Error::ProtocolDrift);
            }

            // Parse ack frame: [8B magic u64 LE][8B token u64 LE].
            // The magic u64 has MAGIC_BUFFER_RELEASED in low-32 bits and 0 in high-32.
            let Ok(magic_bytes) = <[u8; 8]>::try_from(&hdr[ACK_MAGIC_OFFSET..ACK_TOKEN_OFFSET])
            else {
                for idx in dead_indices.into_iter().rev() {
                    subs.remove(idx);
                }
                return Err(Error::Truncated {
                    got: hdr.len(),
                    want: HDR_LEN,
                });
            };
            let magic_u64 = u64::from_le_bytes(magic_bytes);
            // Low-32 bits of the magic_u64 LE value.
            let magic_lo = magic_u64 as u32;
            if magic_lo != MAGIC_BUFFER_RELEASED {
                for idx in dead_indices.into_iter().rev() {
                    subs.remove(idx);
                }
                return Err(Error::BadMagic { got: magic_lo });
            }

            let Ok(token_bytes) = <[u8; 8]>::try_from(&hdr[ACK_TOKEN_OFFSET..HDR_LEN]) else {
                for idx in dead_indices.into_iter().rev() {
                    subs.remove(idx);
                }
                return Err(Error::Truncated {
                    got: hdr.len(),
                    want: HDR_LEN,
                });
            };
            found_token = Some(u64::from_le_bytes(token_bytes));
            break 'outer;
        }

        // Prune dead streams (reverse order to preserve lower indices).
        for idx in dead_indices.into_iter().rev() {
            subs.remove(idx);
        }

        Ok(found_token)
    }
}

impl Drop for LinuxPublisher {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self.accept_thread.take() {
            let _ = handle.join();
        }
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

// ── LinuxSubscriber ───────────────────────────────────────────────────────────

/// UDS subscriber that connects to a publisher socket and polls for incoming
/// fd messages in a non-blocking manner.
pub struct LinuxSubscriber {
    stream: UnixStream,
}

impl LinuxSubscriber {
    /// Connect to `socket_path` (publisher must already be listening).
    pub fn open(socket_path: &str) -> Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        stream.set_nonblocking(true)?;
        Ok(Self { stream })
    }
}

impl FdPassingConnection for LinuxSubscriber {
    fn send_with_fd(&self, _fd: BorrowedFd<'_>, _len: u64, _token: u64) -> Result<()> {
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "subscriber cannot send fds",
        )))
    }

    #[allow(unsafe_code)]
    fn recv_with_fd(&self) -> Result<Option<(OwnedFd, u64, u64)>> {
        // Non-blocking poll(2) with 0ms timeout.
        // SAFETY: poll(2) is a plain Linux syscall; pfd is stack-allocated and
        // valid for the duration of the call.  We use libc directly because
        // rustix does not expose a zero-timeout poll shorthand in v1.x.
        let ready = unsafe {
            let mut pfd = libc::pollfd {
                fd: self.stream.as_fd().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            libc::poll(core::ptr::addr_of_mut!(pfd), 1, 0)
        };

        if ready == 0 {
            // No data available yet.
            return Ok(None);
        }
        if ready < 0 {
            let e = std::io::Error::last_os_error();
            if e.kind() == std::io::ErrorKind::Interrupted {
                return Ok(None);
            }
            return Err(Error::Io(e));
        }

        // Data available — receive the 16-byte header + ancillary fd.
        let mut hdr = [0u8; HDR_LEN];
        let mut iov = [IoSliceMut::new(&mut hdr)];
        let mut space = [core::mem::MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
        let mut cmsg_buf = RecvAncillaryBuffer::new(&mut space);

        let result = rustix::net::recvmsg(
            self.stream.as_fd(),
            &mut iov,
            &mut cmsg_buf,
            RecvFlags::empty(),
        );

        match result {
            Err(e) if e == rustix::io::Errno::AGAIN || e == rustix::io::Errno::WOULDBLOCK => {
                return Ok(None);
            }
            Err(e) if e == rustix::io::Errno::INTR => {
                return Ok(None);
            }
            Err(e) => {
                return Err(Error::Io(std::io::Error::from_raw_os_error(
                    e.raw_os_error(),
                )));
            }
            Ok(msg) => {
                if msg.bytes == 0 {
                    // Peer closed connection.
                    return Err(Error::Disconnected);
                }
                if msg.bytes < HDR_LEN {
                    return Err(Error::Truncated {
                        got: msg.bytes,
                        want: HDR_LEN,
                    });
                }
            }
        }

        // Parse wire v2 forward header.
        // bytes 0..8: payload_len u64 LE
        let Ok(len_bytes) = <[u8; 8]>::try_from(&hdr[PAYLOAD_LEN_OFFSET..TOKEN_OFFSET]) else {
            return Err(Error::Truncated {
                got: hdr.len(),
                want: HDR_LEN,
            });
        };
        let payload_len = u64::from_le_bytes(len_bytes);

        // bytes 8..16: token u64 LE
        let Ok(token_bytes) = <[u8; 8]>::try_from(&hdr[TOKEN_OFFSET..HDR_LEN]) else {
            return Err(Error::Truncated {
                got: hdr.len(),
                want: HDR_LEN,
            });
        };
        let token = u64::from_le_bytes(token_bytes);

        // Extract OwnedFd from SCM_RIGHTS ancillary.
        // If ancillary is absent, this is an ack frame — protocol drift.
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

        match owned_fd {
            Some(fd) => Ok(Some((fd, payload_len, token))),
            None => Err(Error::ProtocolDrift),
        }
    }

    fn send_release_ack(&self, token: u64) -> Result<()> {
        // Build 16-byte ack frame: [magic_u64 LE][token u64 LE].
        // magic_u64 = MAGIC_BUFFER_RELEASED (0x4D4F_5346) in low-32 bits,
        // 0x0000_0000 in high-32 bits → u64 = 0x0000_0000_4D4F_5346.
        let magic_u64: u64 = u64::from(MAGIC_BUFFER_RELEASED); // low-32 = magic, high-32 = 0
        let mut frame = [0u8; HDR_LEN];
        frame[ACK_MAGIC_OFFSET..ACK_TOKEN_OFFSET].copy_from_slice(&magic_u64.to_le_bytes());
        frame[ACK_TOKEN_OFFSET..HDR_LEN].copy_from_slice(&token.to_le_bytes());

        // The stream is non-blocking (set in LinuxSubscriber::open).
        // Use a single write; EAGAIN/EWOULDBLOCK means the send buffer is full —
        // treat as best-effort success (drop the ack silently).
        // A 16-byte frame is smaller than any UDS kernel buffer, so a partial
        // write should never occur in practice, but we guard for it anyway.
        match (&self.stream).write(&frame) {
            Ok(n) if n == frame.len() => Ok(()),
            Ok(_) => Err(Error::Truncated {
                got: 0,
                want: frame.len(),
            }),
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.raw_os_error() == Some(libc::EAGAIN) =>
            {
                // Best-effort: subscriber send buffer full. Drop the ack.
                Ok(())
            }
            Err(e) => Err(Error::Io(e)),
        }
    }

    fn recv_release_ack(&self) -> Result<Option<u64>> {
        // Subscriber role does not receive acks — acks flow to publisher.
        Ok(None)
    }
}

// ── Linux namespace ───────────────────────────────────────────────────────────

/// Namespace struct providing ergonomic constructors.
///
/// ```ignore
/// let pub_ = Linux::open_publisher("/tmp/my.sock")?;
/// let sub  = Linux::open_subscriber("/tmp/my.sock")?;
/// ```
pub struct Linux;

impl Linux {
    /// Open a publisher on `socket_path`.
    pub fn open_publisher(socket_path: &str) -> Result<LinuxPublisher> {
        LinuxPublisher::open(socket_path)
    }

    /// Open a subscriber on `socket_path`.
    pub fn open_subscriber(socket_path: &str) -> Result<LinuxSubscriber> {
        LinuxSubscriber::open(socket_path)
    }
}

// ── peercred check ────────────────────────────────────────────────────────────

/// Validate that the peer has the same effective UID as the current process.
///
/// Uses Linux `SO_PEERCRED` via `getsockopt(2)`.
/// Enabled only when the `peercred` feature is active.
#[cfg(feature = "peercred")]
#[allow(unsafe_code)]
fn check_peer_uid(stream: &UnixStream) -> Result<()> {
    use std::os::fd::AsRawFd as _;

    // SAFETY: getsockopt with SO_PEERCRED is a valid operation on a connected
    // Unix-domain socket fd; `ucred` is a plain C struct with no ownership.
    let cred: libc::ucred = unsafe {
        let mut val: libc::ucred = core::mem::zeroed();
        let mut len = core::mem::size_of::<libc::ucred>() as libc::socklen_t;
        let rc = libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            core::ptr::addr_of_mut!(val) as *mut libc::c_void,
            core::ptr::addr_of_mut!(len),
        );
        if rc != 0 {
            return Err(Error::Io(std::io::Error::last_os_error()));
        }
        val
    };

    // SAFETY: geteuid() is always safe.
    let expected_uid = unsafe { libc::geteuid() };
    if cred.uid != expected_uid {
        return Err(Error::Io(std::io::Error::other(format!(
            "peer UID {peer} != expected {expected_uid}",
            peer = cred.uid,
        ))));
    }
    Ok(())
}
