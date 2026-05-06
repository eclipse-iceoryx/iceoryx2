<!--
TITLE: [#1570] Add iceoryx2-dmabuf with parallel dmabuf::Service variant
SUPERSEDES: PR #1572 (sidecar prototype, will be closed)
CLOSES: #1570
SOURCE: julienzarka/iceoryx2 @ feat/dmabuf-service-variant
TARGET: eclipse-iceoryx/iceoryx2 @ main
-->

# [#1570] Add iceoryx2-dmabuf with parallel dmabuf::Service variant

Closes #1570. Supersedes #1572.

## Summary

This is a redesigned version of #1572 incorporating the maintainer feedback that
DMA-BUF fd-passing "should be possible to implement as a service variant in
iceoryx2." An architecture spike (Task 0a, see commit `c4d719284`) found that
`impl iceoryx2::service::Service for dmabuf::Service` is not viable on stable
Rust 1.85: `payload_start_address` on `cal::SharedMemory` is called at an
infallible call-site in `data_segment.rs:247`, `PointerOffset` is a packed `u64`
that cannot losslessly encode `RawFd + token`, and default associated types
(`type FdConnection`) require `feature(associated_type_defaults)` which is
unstable. `dmabuf::Service` therefore ships as a parallel construct that embeds
`Node<ipc::Service>` for the control-plane (discovery, dynamic config, monitoring,
static storage) and introduces two purpose-built standalone data-plane traits. The
public `Service` trait is untouched; `iceoryx2-cal` gains no new modules.

## What changed vs #1572

| #1572 (sidecar)                                          | This PR (parallel service)                                           |
|----------------------------------------------------------|----------------------------------------------------------------------|
| `FdSidecarPublisher<S, Meta>`                            | `DmaBufServicePublisher<Meta>` (typed alias: `DmaBufPublisher<Meta>`) |
| `FdSidecarToken` in iceoryx2 user-header                 | Internal token in cal-layer wire frame; user-header is caller-owned  |
| Two ports per service (UDS + iceoryx2)                   | One uniform `Service::open_or_create` entry point                    |
| `iceoryx2::port::side_channel::SideChannel` trait in core | Removed - entirely in `iceoryx2-dmabuf`                             |
| ~3K LOC standalone crate                                 | ~1500 LOC: ~600 src + ~280 tests + benchmarks                        |
| iceoryx2-cal untouched                                   | Still untouched - confirmed by spike                                 |

## Architecture overview

Full design rationale lives in:

- `iceoryx2-dmabuf/specs/arch-dmabuf-service-variant.adoc` -- 8 design decisions
  (D1-D8), Mermaid component diagram, post-spike pivot section (why parallel
  construct, not `impl Service`).
- `iceoryx2-dmabuf/specs/spec-dmabuf-service-variant.adoc` -- numbered
  requirements (SVC-1 through SVC-N) with acceptance criteria.

**Key invariant.** Forward fd channel and back ack channel share one bidirectional
`UnixStream`. The presence of an `SCM_RIGHTS` ancillary message disambiguates the
two frame types:

- Forward (publisher to subscriber): `[8B len][8B token][SCM_RIGHTS fd]`
- Back-ack (subscriber to publisher): `[8B magic 0x4D4F5346][8B token]`

No second socket, no out-of-band signalling.

The two standalone data-plane traits introduced are:

- `FdBackedSharedMemory` (`src/shm.rs`) -- fd-backed buffer, `mmap` on open,
  `munmap` on drop; NOT a sub-trait of `cal::SharedMemory`.
- `FdPassingConnection` (`src/connection.rs`) -- fd-passing over a Unix domain
  socket via `SCM_RIGHTS`; NOT a sub-trait of `cal::ZeroCopyConnection`.

Both are `#[cfg(target_os = "linux")]` with `non_linux.rs` stubs so the crate
compiles on Darwin and other platforms.

## Cargo features

| Feature             | Description                                              | Platform   |
|---------------------|----------------------------------------------------------|------------|
| `default = ["std"]` | iceoryx2 std support                                     | All        |
| `dma-buf`           | `DmaBufPublisher` / `DmaBufSubscriber` typed convenience | Linux only |
| `peercred`          | `SO_PEERCRED` UID check on UDS accept                    | Linux only |

`dma-buf` pulls the `dma-buf 0.5` crate. Without it the core
`DmaBufServicePublisher` / `DmaBufServiceSubscriber` API is still available; the
typed `DmaBufPublisher<Meta>` convenience layer is gated.

## Commit structure

12 commits ahead of `upstream/main` (excludes this PR-MESSAGE commit `1485d610c`; re-verify before push — commit list correct as of `1485d610c`):

```
dd0c18ff9 docs: spec + arch + plan for dmabuf service variant
c4d719284 docs: pivot to parallel dmabuf::Service per Task 0a spike findings
48c805d29 feat(iceoryx2-dmabuf): add ExternalFdBuffer + FdBackedSharedMemory
f031d7970 feat(iceoryx2-dmabuf): add FdPassingConnection standalone trait + Linux impl
a21ec5be4 fix(iceoryx2-dmabuf): address review findings on connection + shm
891bd2a25 feat(iceoryx2-dmabuf): add dmabuf::Service parallel construct + port factory
3cc1fb56a feat(iceoryx2-dmabuf): add DmaBufPublisher/Subscriber typed convenience over dmabuf::Service
8fc49e811 feat(iceoryx2-dmabuf): widen wire to v2; add token + back-channel ack
c5c736e2d test(iceoryx2-dmabuf): migrate fd-identity + heap-roundtrip tests
076bcc657 docs(iceoryx2-dmabuf): add service-variant examples + rewrite README
16ce87546 chore(benchmarks): add iceoryx2-benchmarks-dmabuf for service-variant API
7da2d5ee3 docs: mark sidecar specs superseded; update release notes for dmabuf::Service
```

Grouping:

- **2 doc commits** (`dd0c18ff9`, `c4d719284`): initial spec/arch and post-spike
  pivot.
- **4 feat commits** (`48c805d29`, `f031d7970`, `891bd2a25`, `3cc1fb56a`):
  Tasks 1-4 (shm layer, connection layer, service + port factory, typed
  convenience).
- **1 fix commit** (`a21ec5be4`): review findings on connection + shm.
- **4 feat/test/bench/docs commits** (`8fc49e811`, `c5c736e2d`, `076bcc657`,
  `16ce87546`): Tasks 5-7 (wire v2 + ack, test migration, examples + README,
  benchmarks).
- **1 final doc commit** (`7da2d5ee3`): Task 8 sweep (release notes, sidecar
  specs marked superseded).

Each commit compiles and passes `cargo clippy -p iceoryx2-dmabuf` on its own.
`git bisect` between any two adjacent commits stays green per-crate. Pre-existing
workspace-level clippy failures are noted in Known Limitations below and are not
introduced by this PR.

## Test evidence

```
tests/shm_tests.rs          3 tests (Linux)
tests/connection_tests.rs   4 tests including fanout, disconnect, back-channel ack (Linux)
tests/service_tests.rs      2 tests including round-trip (Linux)
tests/it_roundtrip.rs       1 test typed memfd round-trip (Linux + dma-buf)
tests/it_dmabuf_identity.rs 1 test st_ino preserved through SCM_RIGHTS (Linux + dma-buf)
tests/it_dmabuf_heap.rs     1 test real /dev/dma_heap/system + MappedDmaBuf::read (Linux + dma-buf, skips otherwise)
```

All 12 tests pass on Linux x86_64. Darwin compiles cleanly via non-Linux stubs;
0 tests run on Darwin, which is expected.

## Out of scope

- Multi-planar buffers (YUV420 separate Y/UV fds)
- Async transport (tokio / async-std executor integration)
- `sync_file` GPU fence fd passing
- Windows DXGI handle passing
- `pidfd_getfd(2)` as alternative fd-passing mechanism
- Allocator wrappers (`DmaBufPool`)

## Known limitations

**Pre-existing workspace clippy failure:** `iceoryx2-bb-loggers` has mutually
exclusive `buffer` / `file` features that conflict when built with
`--all-features`. This failure pre-dates this PR. Per-crate clippy on
`iceoryx2-dmabuf` is clean (`cargo clippy -p iceoryx2-dmabuf --all-features`).
Same applies to `iceoryx2-pal-testing` watchdog under `--no-default-features`.
Neither failure is introduced by this PR.

## Migration guide for sidecar users

For anyone tracking PR #1572:

- Replace `FdSidecarPublisher<S, Meta>` with `DmaBufServicePublisher<Meta>`. The
  `S` service-type generic is gone; the control-plane is always `ipc::Service`.
- `FdSidecarToken` in the user-header is removed. The token now lives in the
  cal-layer wire frame (`[8B token]` field); your user-header payload is yours
  again with no reserved fields.
- For pool-ack semantics, replace `BackChannel` / `BufferReleased` wire with
  `subscriber.release(token)` / `publisher.recv_release_ack()`.
- For external token assignment (`AckLedger`-style), use `publish_with_token` /
  `receive_with_token`.

## Questions for maintainers

1. **Crate name `iceoryx2-dmabuf`**: keep, or rename to `iceoryx2-fd-passing`?
   The transport is generic (any `RawFd`); DMA-BUF is one application. Renaming
   now is cheaper than later.

2. **`dma-buf` feature default**: currently opt-in. Should it be on by default
   given Linux is the primary target?

3. **Future trait surface**: once `feature(associated_type_defaults)` stabilises,
   should `dmabuf::Service` be retrofitted to satisfy a new
   associated-type-default-friendly variant of the `Service` trait, or should the
   parallel-construct shape remain indefinitely?

4. **Parallel-construct acceptability**: is this shape acceptable as a long-term
   API, or do you prefer deferring the crate until the `Service` trait extension
   surface can accommodate it without UB stubs?

## Checklist

- [x] Issue exists ([#1570](https://github.com/eclipse-iceoryx/iceoryx2/issues/1570))
- [x] Branch prefix `feat/dmabuf-service-variant`
- [x] Commit prefix `[#1570]` or conventional commit
- [x] Eclipse copyright header on every new `.rs` file
- [x] Release notes added
- [x] ECA signed
- [ ] CI pass on Linux x86_64 (pending push)
- [ ] CI pass on aarch64 Linux (pending push)
- [ ] CI pass on Darwin (pending push)
