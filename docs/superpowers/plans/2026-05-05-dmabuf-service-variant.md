# dmabuf::Service Variant Implementation Plan (v1.2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. **All Rust implementation tasks delegate to `mos:rust-coder` agent. All test writing to `mos:tester`. All review checkpoints to `mos:code-reviewer`. All ops (branch / commit / PR) to `mos:ops`. Architecture spikes to `mos:architect`.**

**Goal:** Re-architect the iceoryx2-dmabuf sidecar work as a fully integrated `dmabuf::Service` variant inside iceoryx2, per maintainer roadmap feedback. Add minimal new data-plane concepts in `iceoryx2-dmabuf`, embed `ipc::Service` for control-plane, shrink the standalone crate to a thin typed convenience.

**Architecture (pinned upfront after dry-run v1.0 and Task 0a spike):**

1. **Wire format** — `[8B payload_len LE][8B reserved LE][SCM_RIGHTS ancillary: 1 fd]`. No token in user-header. (Pinned to resolve dry-run issue C; `PointerOffset` dropped per spike.)
2. **Connection trait** — new `FdPassingConnection` is **standalone**, NOT a sub-trait of `ZeroCopyConnection`. No `Service::FdConnection` associated type needed — `dmabuf::Service` is not `impl Service`. (Pinned by Task 0a spike.)
3. **Publisher / subscriber types** — concrete `DmaBufServicePublisher<Meta>` / `DmaBufServiceSubscriber<Meta>`, no `S` generic. (Forced by spike: `impl Service` is not viable.)
4. **NamedConcept** — UDS path derived from service name + port id inside `connection.rs`. One source of truth for naming.
5. **No cal changes** — `iceoryx2-cal` is NOT modified. All new code lives in `iceoryx2-dmabuf`. (Determined by Task 0a spike.)
6. **`dmabuf::Service` struct** — `pub struct Service { control: iceoryx2::service::ipc::Service, ... }` in `iceoryx2-dmabuf/src/service.rs`. NOT `impl crate::service::Service`.

**Tech Stack:** Rust 2024 edition, `rustix 1.x`, `libc`, `dma-buf 0.5` (mripard, optional), `iceoryx2-dmabuf` test harness. Reference branch `feat/dmabuf-sidecar-git-consumable` for proven SCM_RIGHTS / peercred / mmap primitives.

**Task → PR commit mapping (post-spike; 4-5 commits, cal-layer changes removed):**

| Task | PR commit |
|---|---|
| 0a (validation spike) | NOT in PR — internal validation only |
| 0b (branch + scaffolding) | NOT in PR — branch state |
| 1 | commit 1: `feat(iceoryx2-dmabuf): add ExternalFdBuffer + FdBackedSharedMemory` |
| 2 | commit 2: `feat(iceoryx2-dmabuf): add FdPassingConnection standalone trait` |
| 3 | commit 3: `feat(iceoryx2-dmabuf): add dmabuf::Service parallel construct + port factory` |
| 4 + 5 | commit 4: `refactor(iceoryx2-dmabuf): shrink to typed convenience` |
| 6a + 6b | squashed into commit 4 |
| 7 | squashed into commit 4 |
| 8 | squashed into commit 4 |
| 9 | commit 5: `chore(benchmarks): migrate to dmabuf::Service` |
| 10 | commit 5 or standalone: `docs: spec + arch + supersession + release notes` |
| 11 | NOT in PR — PR description only |

---

## Task 0a: Architecture validation spike (mos:architect, then mos:rust-coder)

**Goal:** Before writing 3K+ LOC of cal-layer code, prove the three structural assumptions that drove the dry-run pivots. Read-only investigation + minimal scratch impl. **Outcome: spike/SPIKE-RESULTS.md** documenting decisions.

**Files (scratch, never committed):**
- `/tmp/iox2-spike/` — throwaway crate

- [x] **Step 1: Dispatch mos:architect for trait coupling investigation**

Hand mos:architect the brief: "Read `iceoryx2/src/service/mod.rs` and `ipc.rs`. Determine: (a) Can we add `type FdConnection: FdPassingConnection;` as a NEW associated type on `crate::service::Service`? (b) Does `ResizableSharedMemory` permit `()` or a degenerate impl? (c) Does `Connection` permit a no-op impl that always returns `ConnectionCorrupted`? Report file:line evidence. No code changes."

Expected: 1-page report with go/no-go on each.

- [ ] **Step 2: Dispatch mos:rust-coder for cal-trait impl spike**

Hand mos:rust-coder the brief: "In a scratch crate at /tmp/iox2-spike/, create a minimal `FdBackedSharedMemory` trait + a stub Linux impl. Try to satisfy `iceoryx2_cal::shared_memory::SharedMemory<NoAllocator>` (with the local NoAllocator). Goal: identify which trait methods we can satisfy and which fight us. Compile only — no tests. Report list of methods that need real impls vs unreachable!() stubs."

Expected: list of "must implement" vs "can stub" methods.

- [x] **Step 3: Decide based on spike results**

**SPIKE RESULT: ALL RED — pivot to parallel dmabuf::Service, NOT impl Service.**

Evidence (from mos:architect report):
- `iceoryx2-cal/src/shared_memory/mod.rs:174` (`payload_start_address`) called at `iceoryx2/src/port/details/data_segment.rs:247` for INFALLIBLE offset translation. Degenerate impl = UB.
- `PointerOffset` is a packed `u64` (segment_id + offset). Cannot losslessly encode `RawFd + token`.
- `iceoryx2/src/port/details/sender.rs:180` and `receiver.rs:144` call real SHM ops — no-op impl is not valid.
- Default associated types are unstable on Rust 1.85 / Edition 2024. Adding `type FdConnection` to `Service` breaks 4 existing `impl Service` sites + `examples/rust/service_variant_customization/custom_service_variant.rs:27`.

**Pivot decision:** `dmabuf::Service` is a parallel construct that EMBEDS `ipc::Service` for control-plane delegation. Data-plane lives entirely in `iceoryx2-dmabuf/src/`. No cal-layer changes. The public `Service` trait is untouched.

- [x] **Step 4: Commit decision (text only, no code)**

Recorded in `arch-dmabuf-service-variant.adoc` (§D5, §D1, §Post-spike pivot) and `spec-dmabuf-service-variant.adoc` (§Post-spike scope reduction).

---

## Task 0a Spike Result

**Status:** COMPLETED. Spike found `impl crate::service::Service for dmabuf::Service` is NOT viable.

**Key findings (file:line):**
- `iceoryx2-cal/src/shared_memory/mod.rs:174` — `payload_start_address()` called infallibly; fake address = UB
- `iceoryx2/src/port/details/data_segment.rs:247` — infallible offset translation call site
- `iceoryx2/src/port/details/sender.rs:180` — `try_send` expects real SHM ops
- `iceoryx2/src/port/details/receiver.rs:144` — `release` expects real SHM ops
- `examples/rust/service_variant_customization/custom_service_variant.rs:27` — user-extension example broken by new associated type

**Pivot summary:**
- `dmabuf::Service` = `pub struct Service { control: iceoryx2::service::ipc::Service, ... }` in `iceoryx2-dmabuf/src/service.rs`
- `FdBackedSharedMemory` = standalone trait, `iceoryx2-dmabuf/src/shm.rs` (no cal changes)
- `FdPassingConnection` = standalone trait, `iceoryx2-dmabuf/src/connection.rs` (no cal changes)
- `NoAllocator` / `ShmAllocator` concern dropped; replaced by `ExternalFdBuffer` plain wrapper
- Port types: concrete `DmaBufServicePublisher<Meta>` / `DmaBufServiceSubscriber<Meta>`, no `S` generic
- Task→commit mapping shrinks from 7 commits to 4-5 (cal-layer tasks vanish)

---

## Task 0b: Branch + scaffolding (mos:ops)

Steps unchanged from v1.0 — see below.

---

## File Structure

**Create (revised post-spike — all in `iceoryx2-dmabuf/src/`):**
- `iceoryx2-dmabuf/src/external_buffer.rs` — `ExternalFdBuffer` plain wrapper (replaces `iceoryx2-cal/src/shm_allocator/no_allocator.rs`)
- `iceoryx2-dmabuf/src/shm.rs` — `FdBackedSharedMemory` standalone trait + Linux/non_linux impls (replaces `iceoryx2-cal/src/shared_memory/dmabuf/`)
- `iceoryx2-dmabuf/src/connection.rs` — `FdPassingConnection` standalone trait + Linux/non_linux impls (replaces `iceoryx2-cal/src/zero_copy_connection/dmabuf/`)
- `iceoryx2-dmabuf/src/service.rs` — `dmabuf::Service` struct embedding `ipc::Service` (NOT `impl crate::service::Service`)
- `iceoryx2-dmabuf/tests/{shm_tests,connection_tests,service_tests}.rs`
- `iceoryx2-dmabuf/tests/it_{roundtrip,dmabuf_identity,dmabuf_heap}.rs`
- `iceoryx2-dmabuf/examples/publish_subscribe_dmabuf_service/{publisher,subscriber}.rs`

**Modify:**
- `iceoryx2-dmabuf/src/lib.rs` — drop sidecar mods; add new mods; re-export `DmaBufService`
- `iceoryx2-dmabuf/src/dmabuf_publisher.rs`, `dmabuf_subscriber.rs` — rewrite as newtypes (no S generic)
- `iceoryx2-dmabuf/Cargo.toml` — drop sidecar deps; keep `iceoryx2`, `dma-buf`; drop `iceoryx2-cal` direct dep
- `iceoryx2-dmabuf/specs/arch-fd-sidecar.adoc`, `spec-dmabuf-typed-transport.adoc` — supersession header
- `iceoryx2-dmabuf/README.md` — rewrite for service variant
- `doc/release-notes/iceoryx2-unreleased.md` — Breaking + Features sections
- `benchmarks/dmabuf/src/bench_*.rs` — port to service variant

**NOT modified (spike confirmed):**
- `iceoryx2-cal/src/shm_allocator/` — no changes
- `iceoryx2-cal/src/shared_memory/` — no changes
- `iceoryx2-cal/src/zero_copy_connection/` — no changes
- `iceoryx2/src/service/mod.rs` — no `dmabuf` module added here
- `iceoryx2/src/port/` — no dmabuf port modules added here

**Delete:**
- `iceoryx2-dmabuf/src/{back_channel,error,path,publisher,scm,side_channel,subscriber,token,wire}.rs`
- `iceoryx2-dmabuf/src/bin/{fd_sidecar_crash_pub,fd_sidecar_fd_identity}.rs`
- `iceoryx2-dmabuf/tests/{back_channel,dmabuf_roundtrip,error_paths,it_crash_midsend,it_fanout,it_fd_identity,it_service_gone,it_socket_gone,peercred_mismatch,prop_roundtrip,refcount_survival,unit_dmabuf_publisher,unit_generic_service,unit_path,unit_scm,unit_token}.rs`
- `iceoryx2-dmabuf/examples/{publish_subscribe_with_fd,publish_subscribe_dmabuf}/`

**Reference (read-only, for proven primitives):**
- Sidecar branch `feat/dmabuf-sidecar-git-consumable` files: `iceoryx2-dmabuf/src/scm.rs` (SCM_RIGHTS sendmsg/recvmsg, peercred), `src/path.rs` (UDS path derivation pattern, NOT verbatim — adapt for service-name derivation).

---

## Task 1: ExternalFdBuffer + FdBackedSharedMemory standalone trait (TDD)

> **Revised post-spike:** This task now lives entirely in `iceoryx2-dmabuf/src/`. No cal changes. `NoAllocator` / `ShmAllocator` concern dropped.

**Files:**
- Create: `iceoryx2-dmabuf/src/external_buffer.rs`
- Create: `iceoryx2-dmabuf/src/shm.rs`
- Modify: `iceoryx2-dmabuf/src/lib.rs` — add `pub mod external_buffer; pub mod shm;`
- Test: `iceoryx2-dmabuf/tests/shm_tests.rs`

- [ ] **Step 1: Write failing test for ExternalFdBuffer construction**

Test asserts `ExternalFdBuffer::new(fd, 4096)` constructs and `.len == 4096`. Uses `libc::memfd_create` to obtain a real fd. Gated `#[cfg(target_os = "linux")]`.

- [ ] **Step 2: Write failing test for FdBackedSharedMemory from_owned_fd**

Test creates a memfd via `libc::memfd_create`, sizes it with `ftruncate(4096)`, calls `Linux::from_owned_fd(fd, 4096)`, asserts `shm.len() == 4096` and `shm.payload_ptr() != null`. Imports from `iceoryx2_dmabuf::shm::{linux::Linux, FdBackedSharedMemory}`. Gated `#![cfg(target_os = "linux")]`.

- [ ] **Step 3: Run tests — verify compile failure**

```bash
cargo test -p iceoryx2-dmabuf --test shm_tests
```

Expected: compile error `unresolved import iceoryx2_dmabuf::shm`.

- [ ] **Step 4: Implement ExternalFdBuffer**

Create `iceoryx2-dmabuf/src/external_buffer.rs`:
- `pub struct ExternalFdBuffer { pub fd: OwnedFd, pub len: usize }` — derive `Debug`.
- `impl ExternalFdBuffer { pub fn new(fd: OwnedFd, len: usize) -> Self }`.
- `#[cfg(target_family = "unix")]` or unconditional.

- [ ] **Step 5: Implement FdBackedSharedMemory trait + Linux impl**

Create `iceoryx2-dmabuf/src/shm.rs`:
- Standalone `pub trait FdBackedSharedMemory` (no super-trait bounds on `SharedMemory`).
- `#[cfg(target_os = "linux")] pub mod linux { pub struct Linux { fd: OwnedFd, base: *mut u8, len: usize } }` — `impl Drop` calling `munmap`; `impl FdBackedSharedMemory` calling `mmap` in `from_owned_fd`.
- `#[cfg(not(target_os = "linux"))] pub mod non_linux { pub struct NonLinux; impl FdBackedSharedMemory for NonLinux { ... Err stubs ... } }`.

- [ ] **Step 6: Wire up module exports**

In `iceoryx2-dmabuf/src/lib.rs`, add `pub mod external_buffer; pub mod shm;`.

- [ ] **Step 7: Add remaining tests and run**

- `mmap_payload_writeable_through_pointer`
- `drop_munmaps_and_closes_fd`

```bash
cargo test -p iceoryx2-dmabuf --test shm_tests
cargo clippy -p iceoryx2-dmabuf --all-features --all-targets -- -D warnings
```

Expected: green.

- [ ] **Step 8: Commit**

```bash
git add iceoryx2-dmabuf/src/external_buffer.rs \
        iceoryx2-dmabuf/src/shm.rs \
        iceoryx2-dmabuf/src/lib.rs \
        iceoryx2-dmabuf/tests/shm_tests.rs
git commit -m "feat(iceoryx2-dmabuf): add ExternalFdBuffer + FdBackedSharedMemory standalone trait"
```

---

## Task 2: FdPassingConnection standalone trait + Linux impl (TDD)

> **Revised post-spike:** Standalone trait in `iceoryx2-dmabuf/src/connection.rs`. No cal changes. No `: ZeroCopyConnection` super-bound. No `PointerOffset` in the wire.

**Files:**
- Create: `iceoryx2-dmabuf/src/connection.rs`
- Modify: `iceoryx2-dmabuf/src/lib.rs` — add `pub mod connection;`
- Test: `iceoryx2-dmabuf/tests/connection_tests.rs`

- [ ] **Step 1: Write failing test for in-process roundtrip**

Test:
1. Build UDS path under `/tmp/iox2-test-<pid>.sock`.
2. `Linux::open_publisher(&path)` then sleep 20ms.
3. `Linux::open_subscriber(&path)` then sleep 20ms.
4. `memfd_create` → `OwnedFd`.
5. `publisher.send_with_fd(fd.as_fd(), 4096)`.
6. Sleep 20ms for kernel delivery.
7. `subscriber.recv_with_fd()` returns `Some((_, 4096))`.

- [ ] **Step 2: Run — verify compile failure**

```bash
cargo test -p iceoryx2-dmabuf --test connection_tests
```

Expected: `unresolved import iceoryx2_dmabuf::connection`.

- [ ] **Step 3: Define standalone trait + module facade**

`iceoryx2-dmabuf/src/connection.rs`:
- Standalone `pub trait FdPassingConnection` (no `: ZeroCopyConnection` bound):
  - `fn send_with_fd(&self, fd: BorrowedFd<'_>, len: u64) -> Result<(), SendError>;`
  - `fn recv_with_fd(&self) -> Result<Option<(OwnedFd, u64)>, RecvError>;`
- `#[cfg(target_os = "linux")] pub mod linux;`
- `#[cfg(not(target_os = "linux"))] pub mod non_linux;`

- [ ] **Step 4: Implement Linux variant by transcribing sidecar primitives**

Reference: `git show feat/dmabuf-sidecar-git-consumable:iceoryx2-dmabuf/src/scm.rs`. Transcribe `ScmRightsPublisher` → `Linux::open_publisher` and `ScmRightsSubscriber` → `Linux::open_subscriber`. Wire frame: `[8B len LE][8B reserved LE][SCM_RIGHTS: 1 fd]` (no `PointerOffset`, no token field).

- [ ] **Step 5: Stub non-linux**

`pub struct NonLinux;` — both methods return `Err`.

- [ ] **Step 6: Wire up module export**

In `iceoryx2-dmabuf/src/lib.rs`, add `pub mod connection;`.

- [ ] **Step 7: Run + verify**

```bash
cargo test -p iceoryx2-dmabuf --test connection_tests
```

Expected: `send_recv_memfd_roundtrip` passes.

- [ ] **Step 8: Add fanout / disconnect / peercred tests**

- `fanout_one_pub_three_sub_100_frames`
- `subscriber_disconnect_publisher_prunes`
- `peer_uid_mismatch_rejected` (gated `#[cfg(feature = "peercred")]`)

- [ ] **Step 9: Run + commit**

```bash
cargo test -p iceoryx2-dmabuf --test connection_tests
cargo clippy -p iceoryx2-dmabuf --all-features --all-targets -- -D warnings
git add iceoryx2-dmabuf/src/connection.rs \
        iceoryx2-dmabuf/src/lib.rs \
        iceoryx2-dmabuf/tests/connection_tests.rs
git commit -m "feat(iceoryx2-dmabuf): add FdPassingConnection standalone trait (SCM_RIGHTS over UDS)"
```

---

## Task 3: dmabuf::Service parallel construct + port factory (TDD)

> **Revised post-spike:** `pub struct Service { control: iceoryx2::service::ipc::Service, ... }` in `iceoryx2-dmabuf/src/service.rs`. NOT `impl crate::service::Service`. No changes to `iceoryx2/src/service/mod.rs`.

**Files:**
- Create: `iceoryx2-dmabuf/src/service.rs`
- Modify: `iceoryx2-dmabuf/src/lib.rs` — add `pub mod service;`
- Test: `iceoryx2-dmabuf/tests/service_tests.rs`

- [ ] **Step 1: Write failing test for Service::open_or_create**

Test: `iceoryx2_dmabuf::service::Service::open_or_create("dmabuf/test")` succeeds; drop it. Gated `#![cfg(target_os = "linux")]`.

- [ ] **Step 2: Run — verify compile failure**

```bash
cargo test -p iceoryx2-dmabuf --test service_tests -- service_open_create
```

Expected: `unresolved import iceoryx2_dmabuf::service`.

- [ ] **Step 3: Implement Service struct**

`iceoryx2-dmabuf/src/service.rs`:
- `#![cfg(target_os = "linux")]`
- Doc comment explaining: parallel construct, NOT impl Service, embeds ipc::Service for control-plane.
- `pub struct Service { control: iceoryx2::service::ipc::Service, /* fd-passing transport */ }` deriving `Debug`.
- `impl Service { pub fn open_or_create(name: &ServiceName) -> Result<DmabufPortFactory, ServiceError> }` — delegates name registration / dynamic config to `ipc::Service`; builds `FdPassingConnection::Linux` for data-plane.

- [ ] **Step 4: Implement DmaBufServicePublisher / DmaBufServiceSubscriber**

`iceoryx2-dmabuf/src/dmabuf_publisher.rs` (internal port, no `S` generic):
- `pub struct DmaBufServicePublisher<Meta> { fd_conn: Arc<dyn FdPassingConnection + Send + Sync>, _phantom: PhantomData<Meta> }`
- `pub fn publish(&mut self, meta: Meta, fd: BorrowedFd<'_>, len: u64) -> Result<(), PublishError>`

`iceoryx2-dmabuf/src/dmabuf_subscriber.rs` (symmetrically).

- [ ] **Step 5: Implement port factory**

`DmabufPortFactory` returned by `Service::open_or_create`:
- `pub fn publisher_builder(&self) -> DmaBufServicePublisherBuilder<Meta>`
- `pub fn subscriber_builder(&self) -> DmaBufServiceSubscriberBuilder<Meta>`

- [ ] **Step 6: Add open_or_create + round-trip tests**

Tests:
- `service_open_create_idempotent`
- `publish_receive_memfd_through_dmabuf_service`
- `meta_user_header_is_callee_owned`

- [ ] **Step 7: Run + commit**

```bash
cargo test -p iceoryx2-dmabuf --test service_tests
cargo clippy -p iceoryx2-dmabuf --all-features --all-targets -- -D warnings
git add iceoryx2-dmabuf/src/service.rs \
        iceoryx2-dmabuf/src/dmabuf_publisher.rs \
        iceoryx2-dmabuf/src/dmabuf_subscriber.rs \
        iceoryx2-dmabuf/src/lib.rs \
        iceoryx2-dmabuf/tests/service_tests.rs
git commit -m "feat(iceoryx2-dmabuf): add dmabuf::Service parallel construct + inherent port factory"
```

---

## Task 4: DmaBufPublisher / DmaBufSubscriber typed convenience newtypes (TDD)

> **Renamed from Task 5. Revised post-spike:** Newtypes wrap `DmaBufServicePublisher<Meta>` directly — no `S` generic, no `FdPassingConnection` trait bound on the user-facing type.

**Files:**
- Rewrite: `iceoryx2-dmabuf/src/dmabuf_publisher.rs` (public newtype)
- Rewrite: `iceoryx2-dmabuf/src/dmabuf_subscriber.rs` (public newtype)
- Modify: `iceoryx2-dmabuf/src/lib.rs`

Steps mirror previous Task 6 (crate shrink), with port types updated to drop the `S` generic.

- [ ] **Step 1: Delete sidecar source files** (same as before)

- [ ] **Step 2: Rewrite lib.rs** — drop sidecar mods, add service/shm/connection/external_buffer, export `DmaBufService`.

- [ ] **Step 3: Rewrite dmabuf_publisher.rs as public newtype**

`DmaBufPublisher<Meta>` wraps `DmaBufServicePublisher<Meta>` (no `S` generic):
- `pub fn create(service_name: &str) -> Result<Self, DmaBufError>`
- `pub fn publish(&mut self, meta: Meta, buf: &dma_buf::DmaBuf) -> Result<(), DmaBufError>`

- [ ] **Step 4: Rewrite dmabuf_subscriber.rs symmetrically**

- [ ] **Step 5: Rewrite Cargo.toml**

Drop `sha1_smol`, `iceoryx2-pal-concurrency-sync`, `rustix`, `libc`, `tracing`, `iceoryx2-cal`. Keep `iceoryx2`, optional `dma-buf`.

- [ ] **Step 6: Add it_roundtrip.rs**

- [ ] **Step 7: Run, fmt, clippy**

```bash
cargo fmt -p iceoryx2-dmabuf --check
cargo test -p iceoryx2-dmabuf --features dma-buf
cargo clippy -p iceoryx2-dmabuf --all-features --all-targets -- -D warnings
```

- [ ] **Step 8: Commit**

```bash
git add -u
git commit -m "refactor(iceoryx2-dmabuf): shrink to typed convenience over dmabuf::Service"
```

---

## Task 4b: Back-channel + token integration (mos:rust-coder + mos:tester)

**Goal:** Port the recent pool-ack work (`send_with_token` / `recv_with_token` from commit `ba29fa694`, `BackChannel` / `BufferReleased` from commit `2775fc846`) into the service variant. Keep both PRs merged into ONE upstream PR per user direction.

**Files:**
- Modify: `iceoryx2-dmabuf/src/connection.rs` — add `send_release_ack` / `recv_release_ack` to `FdPassingConnection`
- Modify: `iceoryx2-dmabuf/src/dmabuf_publisher.rs` (internal) — add `publish_with_token`
- Modify: `iceoryx2-dmabuf/src/dmabuf_subscriber.rs` (internal) — add `receive_with_token` + `release`
- Test: `iceoryx2-dmabuf/tests/connection_tests.rs` — add `back_channel_release_roundtrip`

- [ ] **Step 1: Extend FdPassingConnection with back-channel methods**

Add to `FdPassingConnection` trait:
- `fn send_release_ack(&self, token: u64) -> Result<(), SendError>;`
- `fn recv_release_ack(&self) -> Result<Option<u64>, RecvError>;`

Wire frame for ack: `[8B magic 0x4D4F5346 LE][8B token LE]` (16 bytes, per sidecar `wire.rs:11-18`). Bidirectional on same `UnixStream`.

- [ ] **Step 2: Test back-channel roundtrip**

- [ ] **Step 3: Add publish_with_token / receive_with_token in port types**

Wire format v2: `[8B len][8B token][SCM_RIGHTS fd]`.

- [ ] **Step 4: Add release(token) to subscriber + recv_release_ack to publisher**

- [ ] **Step 5: Run + commit**

- [ ] **Step 6: Update spec + arch documents to reflect v2 wire**

Update `spec-dmabuf-service-variant.adoc` §15 wire format and add back-channel requirements. Update `arch-dmabuf-service-variant.adoc` D3 (wire) and D4 (token correlation).

---

## Task 5: Migrate fd-identity + heap roundtrip tests

> **Renumbered from Task 7.**

**Files:**
- Create: `iceoryx2-dmabuf/tests/it_dmabuf_identity.rs`
- Create: `iceoryx2-dmabuf/tests/it_dmabuf_heap.rs`

Steps identical to previous Task 7 — use `DmaBufPublisher` / `DmaBufSubscriber` from revised crate.

- [ ] **Step 1: Write it_dmabuf_identity.rs**
- [ ] **Step 2: Run — must pass on Linux**
- [ ] **Step 3: Write it_dmabuf_heap.rs**
- [ ] **Step 4: Run + commit**

---

## Task 6: Examples + README

> **Renumbered from Task 8.**

Steps identical to previous Task 8.

- [ ] **Step 1: Write publisher example**
- [ ] **Step 2: Write subscriber example**
- [ ] **Step 3: Rewrite README.md**
- [ ] **Step 4: Build examples + commit**

---

## Task 7: Migrate benchmarks

> **Renumbered from Task 9.**

Steps identical to previous Task 9.

---

## Task 8: Mark old specs as superseded + final checks

> **Renumbered from Task 10.**

Steps identical to previous Task 10, with one addition: verify no `iceoryx2-cal` modules were accidentally added.

```bash
# Verify no cal changes
git diff origin/main -- iceoryx2-cal/
# Expected: empty (no cal changes)
```

---

## Task 9: PR draft

> **Renumbered from Task 11.**

Steps identical to previous Task 11.

---

## Self-review notes

**Spec coverage.** Every numbered requirement in `spec-dmabuf-service-variant.adoc` maps to a task. Requirements 1-2 → Task 0; 3-7 → Task 1; 8-12 → Task 1; 13-21 → Task 2; 22-26 → Task 3; 27-31 → Task 3+4; 32-38 → Task 4; 39-40 → Tasks 4+5; 41 → Task 7; 42-48 → Task 8; 49 → distributed across all task commits.

**Type consistency.** `DmaBufServicePublisher<Meta>` / `DmaBufServiceSubscriber<Meta>` (internal, in `iceoryx2-dmabuf/src/`), `DmaBufPublisher<Meta>` / `DmaBufSubscriber<Meta>` (public newtypes) — distinct names, same crate, no `S` generic on either.

**Risk areas (post-spike revision).**
1. *Task 3 ipc::Service embedding.* `ipc::Service` is a struct, not a trait. Embedding it requires understanding its public constructor — likely `NodeBuilder` path. If `ipc::Service` is not directly constructable, `dmabuf::Service` may need to call `node.service_builder(name).open_or_create()` and store the result. Resolve when the type surface is examined.
2. *Task 3 port factory.* Port factory must return concrete `DmaBufServicePublisher<Meta>`. The factory itself needs to be generic over `Meta` at construction time OR use a builder pattern that defers the `Meta` type parameter. Prefer the builder pattern.
3. *Task 4b wire v2.* Extending `FdPassingConnection` with `send_release_ack` / `recv_release_ack` uses the same bidirectional `UnixStream`. Must ensure publisher-side `recv_release_ack` does not block `send_with_fd` — poll/non-blocking read before each send, or a separate drain call. Document choice in commit message.

These risks are tracked as inline notes within the relevant task steps.

---

## Execution Handoff

**Plan updated to v1.2. Reflects Task 0a spike findings: cal-layer changes removed, parallel dmabuf::Service struct in iceoryx2-dmabuf, standalone traits, no impl Service.**

**Two execution options:**

**1. Subagent-Driven (recommended)** — Dispatch fresh subagent per task, review between tasks. Task 3 (Service struct + embedding) is the highest-risk task.

**2. Inline Execution** — Execute tasks in this session using executing-plans.

**Which approach?**
