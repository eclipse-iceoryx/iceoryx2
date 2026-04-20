# Changelog — iceoryx2-dmabuf

All notable changes to this crate follow [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added

- `DmabufPublisher<S, Meta, C>` and `DmabufSubscriber<S, Meta, C>` generic
  over `S: Service` — supports both `ipc::Service` and `local::Service`.
- `DmabufIpcPublisher` / `DmabufIpcSubscriber` convenience type aliases.
- `FdSideChannel` downstream trait extending
  `iceoryx2::port::side_channel::SideChannel` with `send_fd` and
  `recv_fd_matching` for Linux `SCM_RIGHTS` fd passing.
- `ScmRightsPublisher` / `ScmRightsSubscriber` — Linux-only transport
  implementations of `SideChannel` + `FdSideChannel`.
- `peercred` Cargo feature — `SO_PEERCRED` UID filtering on accept.
- `memfd` Cargo feature — `memfd_create` helpers for tests and examples.
- `DmabufError::Iceoryx { kind, msg }` typed error variant replacing
  format-string flattening via `SideChannelIo`.
- `IceoryxErrorKind` enum (`NodeCreate`, `Service`, `PortBuilder`).
- `DmabufToken::from_nonzero` / `as_nonzero` safe constructors; field
  narrowed to `pub(crate)`.
- `DmabufPublisher::create` / `DmabufSubscriber::create` constructors that
  own their iceoryx2 node and service (no pre-built port required).
- `TestGuard` socket-cleanup helper in `tests/common/` for test isolation.
- Benchmarks crate `benchmarks/dmabuf` with `latency`, `throughput`, and
  `fanout` subcommands.
- `examples/publish_subscribe_with_fd/` publisher + subscriber examples.
- Crate-level `//!` rustdoc block with motivation, architecture, quick-start
  doctest, and platform support table.
