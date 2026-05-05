# dmabuf::Service Variant Implementation Plan (v1.1)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. **All Rust implementation tasks delegate to `mos:rust-coder` agent. All test writing to `mos:tester`. All review checkpoints to `mos:code-reviewer`. All ops (branch / commit / PR) to `mos:ops`. Architecture spikes to `mos:architect`.**

**Goal:** Re-architect the iceoryx2-dmabuf sidecar work as a fully integrated `dmabuf::Service` variant inside iceoryx2, per maintainer roadmap feedback. Add minimal new cal-layer concepts plug them into a new `Service` trait impl, shrink the standalone crate to a thin typed convenience.

**Architecture (pinned upfront after dry-run v1.0):**

1. **Wire format** — `[8B payload_len LE][8B meta_size LE][var Meta bytes][SCM_RIGHTS ancillary: 1 fd]`. Meta travels inline, no second channel, no token in user-header. (Pinned to resolve dry-run issue C.)
2. **Connection trait** — new `FdPassingConnection` is **standalone**, NOT a sub-trait of `ZeroCopyConnection`. New `Service::FdConnection` associated type sits alongside `Connection`. (Pinned to resolve dry-run issue D.)
3. **Publisher / subscriber types in cal** — split `LinuxPublisher` / `LinuxSubscriber` (mirror sidecar `ScmRights*` shape). Single `Linux` violates SRP. (Pinned to resolve dry-run issue B.)
4. **NamedConcept** — fd-backed SHM uses UDS-derived name (the connection's socket path is the name). One source of truth for naming. (Pinned to resolve dry-run issue E.)
5. **`ShmAllocatorConfig: Copy` bound** — relax the trait bound (drop `Copy`); document as breaking change. `Arc<OwnedFd>` rejected as DRY violation. (Pinned to resolve dry-run issue G.)
6. **`ResizableSharedMemory` trait shape** — `dmabuf::Service::ResizableSharedMemory = ()` if Service trait allows; else add `Resizable` newtype in Task 2b. Decide via spike in Task 0a.

**Tech Stack:** Rust 2024 edition, `rustix 1.x`, `libc`, `dma-buf 0.5` (mripard, optional), `iceoryx2-cal` testing harness. Reference branch `feat/dmabuf-sidecar-git-consumable` for proven SCM_RIGHTS / peercred / mmap primitives.

**Task → PR commit mapping (resolves dry-run issue I):**

| Task | PR commit |
|---|---|
| 0a (validation spike) | NOT in PR — internal validation only |
| 0b (branch + scaffolding) | NOT in PR — branch state |
| 1 | commit 1: `feat(iceoryx2-cal): add NoAllocator marker` |
| 2 | commit 2: `feat(iceoryx2-cal): add shared_memory::dmabuf flavor` |
| 3 | commit 3: `feat(iceoryx2-cal): add zero_copy_connection::dmabuf flavor` |
| 4 + 5 | commit 4: `feat(iceoryx2): add dmabuf::Service variant + port types` |
| 6a + 6b | commit 5: `refactor(iceoryx2-dmabuf): shrink to typed convenience` |
| 7 | squashed into commit 5 |
| 8 | squashed into commit 5 |
| 9 | commit 6: `chore(benchmarks): migrate to dmabuf::Service` |
| 10 | commit 7: `docs: spec + arch + supersession + release notes` |
| 11 | NOT in PR — PR description only |

---

## Task 0a: Architecture validation spike (mos:architect, then mos:rust-coder)

**Goal:** Before writing 3K+ LOC of cal-layer code, prove the three structural assumptions that drove the dry-run pivots. Read-only investigation + minimal scratch impl. **Outcome: spike/SPIKE-RESULTS.md** documenting decisions.

**Files (scratch, never committed):**
- `/tmp/iox2-spike/` — throwaway crate

- [ ] **Step 1: Dispatch mos:architect for trait coupling investigation**

Hand mos:architect the brief: "Read `iceoryx2/src/service/mod.rs` and `ipc.rs`. Determine: (a) Can we add `type FdConnection: FdPassingConnection;` as a NEW associated type on `crate::service::Service`? (b) Does `ResizableSharedMemory` permit `()` or a degenerate impl? (c) Does `Connection` permit a no-op impl that always returns `ConnectionCorrupted`? Report file:line evidence. No code changes."

Expected: 1-page report with go/no-go on each.

- [ ] **Step 2: Dispatch mos:rust-coder for cal-trait impl spike**

Hand mos:rust-coder the brief: "In a scratch crate at /tmp/iox2-spike/, create a minimal `FdBackedSharedMemory` trait + a stub Linux impl. Try to satisfy `iceoryx2_cal::shared_memory::SharedMemory<NoAllocator>` (with the local NoAllocator). Goal: identify which trait methods we can satisfy and which fight us. Compile only — no tests. Report list of methods that need real impls vs unreachable!() stubs."

Expected: list of "must implement" vs "can stub" methods.

- [ ] **Step 3: Decide based on spike results**

If both spikes are green: proceed with plan as-is.
If either spike is red: pivot to "parallel dmabuf::Service not impl Service" — record decision in `arch-dmabuf-service-variant.adoc:section "Post-spike pivots"`.

- [ ] **Step 4: Commit decision (text only, no code)**

```bash
# update arch doc with spike results
git add iceoryx2-dmabuf/specs/arch-dmabuf-service-variant.adoc
git commit -m "docs(arch): record spike results pinning Design C cal-layer shape"
```

---

## Task 0b: Branch + scaffolding (mos:ops)

Steps unchanged from v1.0 — see below.

---

## File Structure

**Create:**
- `iceoryx2-cal/src/shm_allocator/no_allocator.rs` — `NoAllocator` + `NoAllocatorConfig`
- `iceoryx2-cal/src/shared_memory/dmabuf/{mod,linux,non_linux}.rs` — `FdBackedSharedMemory` trait + impls
- `iceoryx2-cal/src/zero_copy_connection/dmabuf/{mod,linux,non_linux}.rs` — `FdPassingConnection` trait + impls
- `iceoryx2-cal/tests/{no_allocator_tests,shared_memory_dmabuf_tests,zero_copy_connection_dmabuf_tests}.rs`
- `iceoryx2/src/service/dmabuf.rs` — `dmabuf::Service` struct + Service trait impl
- `iceoryx2/src/port/{dmabuf_publisher,dmabuf_subscriber}.rs` — port types
- `iceoryx2/src/service/port_factory/dmabuf_publish_subscribe.rs` — port factory
- `iceoryx2/tests/service_dmabuf_publish_subscribe_tests.rs`
- `iceoryx2-dmabuf/examples/publish_subscribe_dmabuf_service/{publisher,subscriber}.rs`
- `iceoryx2-dmabuf/tests/{it_dmabuf_identity,it_dmabuf_heap,it_roundtrip}.rs`

**Modify:**
- `iceoryx2-cal/src/shm_allocator/mod.rs` — re-export `NoAllocator`, add `ExternallyAllocated` variant
- `iceoryx2-cal/src/shared_memory/mod.rs` — `pub mod dmabuf;`
- `iceoryx2-cal/src/zero_copy_connection/mod.rs` — `pub mod dmabuf;`
- `iceoryx2/src/service/mod.rs` — `#[cfg(target_os = "linux")] pub mod dmabuf;`
- `iceoryx2/src/port/mod.rs` — add publisher / subscriber dmabuf modules
- `iceoryx2-dmabuf/src/lib.rs` — drop sidecar mods
- `iceoryx2-dmabuf/src/dmabuf_publisher.rs`, `dmabuf_subscriber.rs` — rewrite as newtypes
- `iceoryx2-dmabuf/Cargo.toml` — drop sidecar deps; keep `iceoryx2`, `dma-buf`
- `iceoryx2-dmabuf/specs/arch-fd-sidecar.adoc`, `spec-dmabuf-typed-transport.adoc` — supersession header
- `iceoryx2-dmabuf/README.md` — rewrite for service variant
- `doc/release-notes/iceoryx2-unreleased.md` — Breaking + Features sections
- `benchmarks/dmabuf/src/bench_*.rs` — port to service variant

**Delete:**
- `iceoryx2-dmabuf/src/{back_channel,error,path,publisher,scm,side_channel,subscriber,token,wire}.rs`
- `iceoryx2-dmabuf/src/bin/{fd_sidecar_crash_pub,fd_sidecar_fd_identity}.rs`
- `iceoryx2-dmabuf/tests/{back_channel,dmabuf_roundtrip,error_paths,it_crash_midsend,it_fanout,it_fd_identity,it_service_gone,it_socket_gone,peercred_mismatch,prop_roundtrip,refcount_survival,unit_dmabuf_publisher,unit_generic_service,unit_path,unit_scm,unit_token}.rs`
- `iceoryx2-dmabuf/examples/{publish_subscribe_with_fd,publish_subscribe_dmabuf}/`

**Reference (read-only, for proven primitives):**
- Sidecar branch `feat/dmabuf-sidecar-git-consumable` files: `iceoryx2-dmabuf/src/scm.rs` (SCM_RIGHTS sendmsg/recvmsg, peercred), `src/path.rs` (UDS path derivation pattern, NOT verbatim — use cal NamedConcept instead).

---

## Task 1: NoAllocator marker (TDD)

**Files:**
- Create: `iceoryx2-cal/src/shm_allocator/no_allocator.rs`
- Modify: `iceoryx2-cal/src/shm_allocator/mod.rs:14` — add `pub mod no_allocator;` and `pub use no_allocator::*;`
- Modify: `iceoryx2-cal/src/shm_allocator/mod.rs` — add `ExternallyAllocated` variant to `ShmAllocationError`'s `enum_gen!`
- Test: `iceoryx2-cal/tests/no_allocator_tests.rs`

- [ ] **Step 1: Write failing test for NoAllocatorConfig construction**

Test asserts `NoAllocatorConfig { fd, len: 4096 }` constructs and `cfg.len == 4096`. Uses `libc::memfd_create` to obtain a real fd. Test file is gated `#[cfg(target_os = "linux")]`. Imports `NoAllocator`, `NoAllocatorConfig`, `ShmAllocator` from `iceoryx2_cal::shm_allocator`.

- [ ] **Step 2: Run test — verify it fails (compile error)**

```bash
cargo test -p iceoryx2-cal --test no_allocator_tests
```

Expected: compile error `unresolved import iceoryx2_cal::shm_allocator::NoAllocator`.

- [ ] **Step 3: Add ExternallyAllocated variant to ShmAllocationError**

In `iceoryx2-cal/src/shm_allocator/mod.rs`, find the `enum_gen! { ShmAllocationError ... }` block and add `ExternallyAllocated` to the `entry:` list (comma-separated after `ExceedsMaxSupportedAlignment`).

- [ ] **Step 4: Implement NoAllocator (Linux only)**

Create `iceoryx2-cal/src/shm_allocator/no_allocator.rs` with:
- `pub struct NoAllocatorConfig { pub fd: OwnedFd, pub len: usize }` — derive `Debug`; impl `Clone` (via `fd.try_clone()`) and `Default` (via `memfd_create`); impl `ShmAllocatorConfig`.
- `pub struct NoAllocator { fd: OwnedFd, len: usize }` — derive `Debug`; impl `ShmAllocator` with `type Configuration = NoAllocatorConfig`, `allocate` returning `Err(ShmAllocationError::ExternallyAllocated)`, `max_alignment` returning `4096`, `total_size` returning `self.len`.
- All gated `#[cfg(target_os = "linux")]`.

Note: if `ShmAllocatorConfig: Copy` blocks compilation (`OwnedFd: !Copy`), drop the `Copy` bound from the trait in `mod.rs` and document the breaking change. Decide via `cargo check` once `mod.rs` is wired up.

- [ ] **Step 5: Wire up module exports**

In `iceoryx2-cal/src/shm_allocator/mod.rs`, after the existing `pub mod` lines (line 14), add:

```rust
pub mod no_allocator;
pub use no_allocator::*;
```

- [ ] **Step 6: Add second test asserting allocate fails**

Append a second `#[test]`: construct `NoAllocator { fd, len: 4096 }`, call `alloc.allocate(Layout::new::<u64>())`, assert `matches!(res, Err(ShmAllocationError::ExternallyAllocated))`.

- [ ] **Step 7: Run tests + clippy**

```bash
cargo test -p iceoryx2-cal --test no_allocator_tests
cargo clippy -p iceoryx2-cal --all-features --all-targets -- -D warnings
```

Expected: tests pass, clippy clean.

- [ ] **Step 8: Commit**

```bash
git add iceoryx2-cal/src/shm_allocator/no_allocator.rs \
        iceoryx2-cal/src/shm_allocator/mod.rs \
        iceoryx2-cal/tests/no_allocator_tests.rs
git commit -m "feat(iceoryx2-cal): add NoAllocator marker for externally-allocated SHM"
```

---

## Task 2: shared_memory::dmabuf trait + Linux impl (TDD)

**Files:**
- Create: `iceoryx2-cal/src/shared_memory/dmabuf/{mod,linux,non_linux}.rs`
- Modify: `iceoryx2-cal/src/shared_memory/mod.rs:59-63` — add `pub mod dmabuf;`
- Test: `iceoryx2-cal/tests/shared_memory_dmabuf_tests.rs`

- [ ] **Step 1: Write failing test for from_owned_fd**

Test creates a memfd via `libc::memfd_create`, sizes it with `ftruncate(4096)`, calls `Linux::<NoAllocator>::from_owned_fd(fd, 4096)`, asserts `shm.size() == 4096` and `shm.payload_start_address() != 0`. Imports from `iceoryx2_cal::shared_memory::dmabuf::{linux::Linux, FdBackedSharedMemory}` + `iceoryx2_cal::shm_allocator::NoAllocator`. Gated `#![cfg(target_os = "linux")]`.

- [ ] **Step 2: Run test — verify compile failure**

```bash
cargo test -p iceoryx2-cal --test shared_memory_dmabuf_tests
```

Expected: `unresolved import iceoryx2_cal::shared_memory::dmabuf`.

- [ ] **Step 3: Define the trait + module facade**

`iceoryx2-cal/src/shared_memory/dmabuf/mod.rs`:
- Doc comment: "Fd-backed shared memory: external `OwnedFd`-supplied buffer; mmap on construction, munmap on drop."
- `#[cfg(target_os = "linux")] pub mod linux;`
- `#[cfg(not(target_os = "linux"))] pub mod non_linux;`
- `pub trait FdBackedSharedMemory: SharedMemory<NoAllocator>`:
  - `fn from_owned_fd(fd: OwnedFd, len: usize) -> Result<Self, SharedMemoryCreateError>;`
  - `fn as_fd(&self) -> BorrowedFd<'_>;`

- [ ] **Step 4: Implement Linux variant**

`iceoryx2-cal/src/shared_memory/dmabuf/linux.rs`:
- `#![cfg(target_os = "linux")]`
- `pub struct Linux<A> { fd: OwnedFd, base: *mut u8, len: usize, _phantom: PhantomData<A> }`.
- `unsafe impl Send for Linux<NoAllocator> {}` — backing memory is shared, fd is `Send`. Not `Sync`.
- `impl Drop for Linux<NoAllocator>` — calls `libc::munmap(self.base.cast(), self.len)`.
- `impl FdBackedSharedMemory for Linux<NoAllocator>`:
  - `from_owned_fd`: calls `libc::mmap(NULL, len, PROT_READ|PROT_WRITE, MAP_SHARED, fd.as_raw_fd(), 0)`. If `MAP_FAILED`, returns `Err(SharedMemoryCreateError::InternalError)`. Otherwise wraps base ptr.
  - `as_fd`: returns `self.fd.as_fd()`.
- `unsafe`-allow at module level via `#[allow(unsafe_code)]` because mmap/munmap are the whole point.

Note: full `SharedMemory<NoAllocator>` impl requires `NamedConcept` plumbing. Use the shape in `iceoryx2-cal/src/shared_memory/posix.rs` as a template. The fd-backed SHM has no name in the kernel namespace — derive a stable name from the UDS connection path. If too complex, split into Task 2a (trait + skeleton) and Task 2b (full SharedMemory wiring).

- [ ] **Step 5: Stub non-linux**

`iceoryx2-cal/src/shared_memory/dmabuf/non_linux.rs`:
- `#![cfg(not(target_os = "linux"))]`
- `pub struct NonLinux<A> { _phantom: PhantomData<A> }`
- `impl FdBackedSharedMemory for NonLinux<NoAllocator>`:
  - `from_owned_fd` returns `Err(SharedMemoryCreateError::InternalError)`.
  - `as_fd` is `unreachable!()` (since the type cannot be constructed via the only constructor).

- [ ] **Step 6: Wire up module export**

In `iceoryx2-cal/src/shared_memory/mod.rs`, after line 63 (`pub mod recommended;`), add `pub mod dmabuf;`.

- [ ] **Step 7: Run tests — verify they pass**

```bash
cargo test -p iceoryx2-cal --test shared_memory_dmabuf_tests
```

Expected: `from_owned_fd_with_memfd_succeeds` passes.

- [ ] **Step 8: Add remaining tests**

- `mmap_payload_writeable_through_pointer`: memfd + ftruncate, from_owned_fd, write through `payload_start_address`, map again from a fresh fd, verify byte-identical via the second mapping.
- `drop_munmaps_and_closes_fd`: construct, drop, assert fd is closed via `/proc/self/fd` inspection.

- [ ] **Step 9: Run + commit**

```bash
cargo test -p iceoryx2-cal --test shared_memory_dmabuf_tests
cargo clippy -p iceoryx2-cal --all-features --all-targets -- -D warnings
git add iceoryx2-cal/src/shared_memory/dmabuf/ \
        iceoryx2-cal/src/shared_memory/mod.rs \
        iceoryx2-cal/tests/shared_memory_dmabuf_tests.rs
git commit -m "feat(iceoryx2-cal): add shared_memory::dmabuf flavor (fd-backed SHM)"
```

---

## Task 3: zero_copy_connection::dmabuf trait + Linux impl (TDD)

**Files:**
- Create: `iceoryx2-cal/src/zero_copy_connection/dmabuf/{mod,linux,non_linux}.rs`
- Modify: `iceoryx2-cal/src/zero_copy_connection/mod.rs` — add `pub mod dmabuf;`
- Test: `iceoryx2-cal/tests/zero_copy_connection_dmabuf_tests.rs`

- [ ] **Step 1: Write failing test for in-process roundtrip**

Test:
1. Build UDS path under `/tmp/iox2-test-<pid>.sock`.
2. `Linux::open_publisher(&path)` then sleep 20ms.
3. `Linux::open_subscriber(&path)` then sleep 20ms.
4. `memfd_create` → `OwnedFd`.
5. `publisher.send_with_fd(PointerOffset::new(0), fd.as_fd(), 4096)`.
6. Sleep 20ms for kernel delivery.
7. `subscriber.recv_with_fd()` returns `Some((_, _, 4096))`.

- [ ] **Step 2: Run — verify compile failure**

```bash
cargo test -p iceoryx2-cal --test zero_copy_connection_dmabuf_tests
```

Expected: `unresolved import iceoryx2_cal::zero_copy_connection::dmabuf`.

- [ ] **Step 3: Define trait + module facade**

`iceoryx2-cal/src/zero_copy_connection/dmabuf/mod.rs`:
- Doc comment.
- `#[cfg(target_os = "linux")] pub mod linux;`
- `#[cfg(not(target_os = "linux"))] pub mod non_linux;`
- `pub trait FdPassingConnection: ZeroCopyConnection`:
  - `fn send_with_fd(&self, offset: PointerOffset, fd: BorrowedFd<'_>, len: u64) -> Result<(), ZeroCopySendError>;`
  - `fn recv_with_fd(&self) -> Result<Option<(PointerOffset, OwnedFd, u64)>, ZeroCopyReceiveError>;`

- [ ] **Step 4: Implement Linux variant by transcribing sidecar primitives**

Reference: `git show feat/dmabuf-sidecar-git-consumable:iceoryx2-dmabuf/src/scm.rs` (lines 186-654 cover the Linux impl). Transcribe:
- `ScmRightsPublisher::new` (UDS bind, accept thread, broken-pipe pruning) → `Linux::open_publisher`
- `ScmRightsPublisher::send_fd_impl` → `Linux::send_with_fd` — but replace token-bytes iov with `[len 8B][reserved 8B]` per spec §17
- `ScmRightsSubscriber::new` (UDS connect, set_nonblocking) → `Linux::open_subscriber`
- `ScmRightsSubscriber::recv_fd_matching_impl` (poll + recvmsg) → `Linux::recv_with_fd` — drop the token-matching loop because correlation is implicit per-message; just pull one frame
- `check_peer_uid` → keep verbatim, gated on cal-feature `peercred`

`Linux` struct fields: `socket_path: String`, `listener: Option<UnixListener>`, `subscribers: Arc<Mutex<Vec<UnixStream>>>`, `stream: Option<UnixStream>`, `shutdown: Arc<AtomicBool>`, `accept_thread: Option<JoinHandle<()>>`.

`open_publisher`:
1. `std::fs::remove_file(socket_path)` (ignore error — file may not exist).
2. `UnixListener::bind(socket_path)?`.
3. Create `subscribers: Arc<Mutex<Vec<UnixStream>>>`, `shutdown: Arc<AtomicBool>`.
4. Clone listener via `try_clone`, spawn accept thread that loops while `!shutdown.load(Relaxed)`: accept connections, push into `subs`. EAGAIN → sleep 5ms.
5. Return `Self` with all fields populated.

`open_subscriber`:
1. `UnixStream::connect(socket_path)?`.
2. `stream.set_nonblocking(true)?`.
3. Return `Self` with `stream: Some(...)`, listener / subscribers empty.

`send_with_fd`:
1. Build 16-byte header: `header[0..8] = len.to_le_bytes()`, `header[8..16] = 0`.
2. Lock `subscribers`. For each stream, call `rustix::net::sendmsg` with `iov = [IoSlice::new(&header)]` and SCM_RIGHTS ancillary carrying the fd. Retain successful ones.

`recv_with_fd`:
1. Take `self.stream.as_ref()`.
2. Build 16-byte header receive buffer + 1-fd ancillary buffer.
3. Call `rustix::net::recvmsg`. On `Errno::AGAIN` return `Ok(None)`. On other error return `ZeroCopyReceiveError::ReceiveBufferIsEmpty`.
4. Parse `len = u64::from_le_bytes(header[0..8])`.
5. Drain ancillary, extract `OwnedFd`.
6. Return `Ok(Some((PointerOffset::new(0), fd, len)))`.

`Drop`:
1. `shutdown.store(true, Relaxed)`.
2. If publisher (listener present), `remove_file(socket_path)`.
3. Join accept thread.

Allow `unsafe_code` only for the recvmsg fd extraction (none in this skeleton — `rustix` wraps it safely).

- [ ] **Step 5: Stub non-linux**

`iceoryx2-cal/src/zero_copy_connection/dmabuf/non_linux.rs`:
- `pub struct NonLinux;`
- `impl FdPassingConnection for NonLinux`: both methods return `Err`.

- [ ] **Step 6: Wire up module export**

In `iceoryx2-cal/src/zero_copy_connection/mod.rs`, after the existing `pub mod` lines (~line 18), add `pub mod dmabuf;`.

- [ ] **Step 7: Run + verify**

```bash
cargo test -p iceoryx2-cal --test zero_copy_connection_dmabuf_tests
```

Expected: `send_recv_memfd_roundtrip` passes.

- [ ] **Step 8: Add fanout / disconnect / peercred tests**

- `fanout_one_pub_three_sub_100_frames`: bind one publisher, connect three subscribers, send 100 frames, assert each subscriber receives 100.
- `subscriber_disconnect_publisher_prunes`: connect 2 subscribers, drop one, send a frame, assert publisher's `subscribers` Vec drops to 1.
- `peer_uid_mismatch_rejected` (gated `#[cfg(feature = "peercred")]`): use a child process running under different UID; assert publisher rejects connection.

Reference: sidecar branch's `iceoryx2-dmabuf/tests/it_fanout.rs` and `peercred_mismatch.rs` for the test bodies — adapt to use `Linux::open_publisher` / `Linux::open_subscriber` and the new wire format.

- [ ] **Step 9: Run + commit**

```bash
cargo test -p iceoryx2-cal --test zero_copy_connection_dmabuf_tests
cargo clippy -p iceoryx2-cal --all-features --all-targets -- -D warnings
git add iceoryx2-cal/src/zero_copy_connection/dmabuf/ \
        iceoryx2-cal/src/zero_copy_connection/mod.rs \
        iceoryx2-cal/tests/zero_copy_connection_dmabuf_tests.rs
git commit -m "feat(iceoryx2-cal): add zero_copy_connection::dmabuf flavor"
```

---

## Task 4: dmabuf::Service trait impl (TDD)

**Files:**
- Create: `iceoryx2/src/service/dmabuf.rs`
- Modify: `iceoryx2/src/service/mod.rs` — add `#[cfg(target_os = "linux")] pub mod dmabuf;`
- Test: `iceoryx2/tests/service_dmabuf_publish_subscribe_tests.rs`

- [ ] **Step 1: Write failing test for NodeBuilder construction**

Test: `NodeBuilder::new().create::<iceoryx2::service::dmabuf::Service>()` succeeds; drop it. Gated `#![cfg(target_os = "linux")]`.

- [ ] **Step 2: Run — verify compile failure**

```bash
cargo test -p iceoryx2 --test service_dmabuf_publish_subscribe_tests -- node_builder_create
```

Expected: `unresolved import iceoryx2::service::dmabuf`.

- [ ] **Step 3: Implement dmabuf::Service**

`iceoryx2/src/service/dmabuf.rs`:
- `#![cfg(target_os = "linux")]`
- Doc comment explaining the service variant.
- `pub struct Service {}` deriving `Debug, Clone`.
- `impl crate::service::Service for Service` — copies `ipc::Service`'s associated types verbatim except:
  - `type SharedMemory = dmabuf_shm::linux::Linux<NoAllocator>;`
  - `type ResizableSharedMemory = dmabuf_shm::linux::Linux<NoAllocator>;`
  - `type Connection = dmabuf_conn::linux::Linux;`
- `impl crate::service::internal::ServiceInternal<Service> for Service {}`

Reference: `iceoryx2/src/service/ipc.rs` for the full Service impl shape.

- [ ] **Step 4: Wire up module export**

In `iceoryx2/src/service/mod.rs`, find the existing variant module section (next to `pub mod ipc;`) and add `#[cfg(target_os = "linux")] pub mod dmabuf;`.

- [ ] **Step 5: Run — verify test passes (or surface trait-bound failures)**

```bash
cargo test -p iceoryx2 --test service_dmabuf_publish_subscribe_tests -- node_builder_create
```

Expected: passes. If trait bounds fail (e.g., `ResizableSharedMemory` requires `recommended::Ipc<A>` shape we don't satisfy), the type aliases need to point at a `dmabuf_shm::linux::Resizable` companion type. Add that companion in Task 2 if surfaced here.

- [ ] **Step 6: Add open_or_create test**

Test: build a node, build a service via `service_builder(&"dmabuf/test".try_into()?)` → `publish_subscribe::<u64>()` → `open_or_create()`, drop. No assertions beyond no panic.

- [ ] **Step 7: Run + commit**

```bash
cargo test -p iceoryx2 --test service_dmabuf_publish_subscribe_tests
cargo clippy -p iceoryx2 --all-features --all-targets -- -D warnings
git add iceoryx2/src/service/dmabuf.rs \
        iceoryx2/src/service/mod.rs \
        iceoryx2/tests/service_dmabuf_publish_subscribe_tests.rs
git commit -m "feat(iceoryx2): add dmabuf::Service variant"
```

---

## Task 5: DmaBufServicePublisher / Subscriber port types (TDD)

**Files:**
- Create: `iceoryx2/src/port/dmabuf_publisher.rs`
- Create: `iceoryx2/src/port/dmabuf_subscriber.rs`
- Modify: `iceoryx2/src/port/mod.rs` — `pub mod dmabuf_publisher; pub mod dmabuf_subscriber;`
- Modify: `iceoryx2/src/service/port_factory/mod.rs` + add `dmabuf_publish_subscribe.rs` factory
- Test: `iceoryx2/tests/service_dmabuf_publish_subscribe_tests.rs`

- [ ] **Step 1: Write failing test for round-trip via service**

Test:
1. Build node + service for `dmabuf::Service`, publish_subscribe::<u64>, open_or_create.
2. Build publisher + subscriber from the factory.
3. memfd_create + ftruncate(4096) → OwnedFd.
4. `pubr.publish(42u64, fd.as_fd(), 4096)`.
5. Sleep 20ms.
6. `subr.receive()` returns `Some((42, _fd, 4096))`.

- [ ] **Step 2: Run — verify failure**

```bash
cargo test -p iceoryx2 --test service_dmabuf_publish_subscribe_tests -- publish_receive
```

Expected: type mismatch — `Publisher::publish` doesn't take fd args.

- [ ] **Step 3: Implement DmaBufServicePublisher**

`iceoryx2/src/port/dmabuf_publisher.rs`:
- `pub struct DmaBufServicePublisher<S, Meta> { fd_conn: Arc<S::Connection>, _phantom: PhantomData<Meta> }` bounded on `S: Service<Connection: FdPassingConnection>` + `Meta: ZeroCopySend + Debug + Copy + 'static`.
- `pub(crate) fn new(fd_conn: Arc<S::Connection>) -> Self`.
- `pub fn publish(&mut self, _meta: Meta, fd: BorrowedFd<'_>, len: u64) -> Result<(), PublishError>`:
  - For initial impl, send_with_fd carries (offset=0, fd, len). Meta delivery is added in a follow-up commit if the generic Publisher cannot be re-used.
  - Maps `ZeroCopySendError` → `PublishError::SendError`.

Implementation note: `Publisher<S, Meta, ()>` doesn't expose its connection. Three options:
- (a) Add `pub(crate)` accessor on `Publisher` for the connection.
- (b) Create a separate `dmabuf::Service`-only Publisher inside iceoryx2 that owns the connection directly, bypassing the generic Publisher (preferred — cleanest separation).
- (c) Use the port factory to clone an `Arc<S::Connection>` into the dmabuf publisher.

Recommend (b). The skeleton above uses (b).

- [ ] **Step 4: Implement DmaBufServiceSubscriber symmetrically**

`iceoryx2/src/port/dmabuf_subscriber.rs`:
- Mirror Publisher: `pub struct DmaBufServiceSubscriber<S, Meta> { ... }`.
- `pub fn receive(&mut self) -> Result<Option<(Meta, OwnedFd, u64)>, ReceiveError>`:
  - Calls `recv_with_fd` on the connection.
  - Wraps into `(Meta, OwnedFd, len)`. Meta defaults to a zeroed value until Meta is plumbed through the wire frame.

- [ ] **Step 5: Implement port factory**

`iceoryx2/src/service/port_factory/dmabuf_publish_subscribe.rs`:
- `pub struct PortFactoryDmabuf<S: Service, Meta> { /* connection arc + service config */ }`.
- `impl<S, Meta> PortFactoryDmabuf<S, Meta>` where `S: Service<Connection: FdPassingConnection>`:
  - `pub fn publisher_builder(&self) -> DmaBufServicePublisherBuilder<S, Meta>`.
  - `pub fn subscriber_builder(&self) -> DmaBufServiceSubscriberBuilder<S, Meta>`.

- [ ] **Step 6: Specialise dmabuf::Service's port factory return type**

The `service_builder().publish_subscribe::<Meta>().open_or_create()` chain must, when `S = dmabuf::Service`, return `PortFactoryDmabuf` instead of the generic `PortFactoryPublishSubscribe`. Use a private `Service` trait associated type to dispatch — likely a `type PortFactoryPublishSubscribe<Meta>;` extension to the `Service` trait. Document the trait extension as part of this task.

- [ ] **Step 7: Run + verify roundtrip**

```bash
cargo test -p iceoryx2 --test service_dmabuf_publish_subscribe_tests
```

Expected: passes.

- [ ] **Step 8: Add fd-identity test**

Test: memfd_create, ftruncate, write a known pattern. Publish through `dmabuf::Service`. Receive, fstat, assert same `st_ino` as publisher's fstat (proves fd identity preserved across SCM_RIGHTS).

- [ ] **Step 9: Commit**

```bash
cargo test -p iceoryx2 --test service_dmabuf_publish_subscribe_tests
cargo clippy -p iceoryx2 --all-features --all-targets -- -D warnings
git add iceoryx2/src/port/dmabuf_publisher.rs \
        iceoryx2/src/port/dmabuf_subscriber.rs \
        iceoryx2/src/port/mod.rs \
        iceoryx2/src/service/port_factory/dmabuf_publish_subscribe.rs \
        iceoryx2/src/service/port_factory/mod.rs \
        iceoryx2/tests/service_dmabuf_publish_subscribe_tests.rs
git commit -m "feat(iceoryx2): add DmaBufService publisher/subscriber port types"
```

---


## Task 5b: Back-channel + token integration (mos:rust-coder + mos:tester)

**Goal:** Port the recent pool-ack work (`send_with_token` / `recv_with_token` from commit `ba29fa694`, `BackChannel` / `BufferReleased` from commit `2775fc846`) into the service variant. Keep both PRs merged into ONE upstream PR per user direction.

**Files:**
- Modify: `iceoryx2-cal/src/zero_copy_connection/dmabuf/{mod,linux}.rs` — add `send_release_ack` / `recv_release_ack` to `FdPassingConnection`
- Modify: `iceoryx2/src/port/dmabuf_publisher.rs` — add `publish_with_token(meta, fd, len, token: u64)`
- Modify: `iceoryx2/src/port/dmabuf_subscriber.rs` — add `receive_with_token() -> (Meta, OwnedFd, u64, len)` + `release(token: u64)` for ack-back
- Test: `iceoryx2-cal/tests/zero_copy_connection_dmabuf_tests.rs` — add `back_channel_release_roundtrip`

- [ ] **Step 1: Dispatch mos:rust-coder to extend FdPassingConnection trait**

Brief: "Extend the `FdPassingConnection` trait in `iceoryx2-cal/src/zero_copy_connection/dmabuf/mod.rs` with two methods:
- `fn send_release_ack(&self, token: u64) -> Result<(), ZeroCopySendError>;` — consumer→producer wire write of `BufferReleased { magic: 0x4D4F5346, token }`. Best-effort (EAGAIN → Ok).
- `fn recv_release_ack(&self) -> Result<Option<u64>, ZeroCopyReceiveError>;` — non-blocking read of one `BufferReleased`. Wire frame is 16 bytes per `iceoryx2-dmabuf/src/wire.rs:11-18` on the sidecar branch.

Reference shape: `git show feat/dmabuf-sidecar-git-consumable:iceoryx2-dmabuf/src/back_channel.rs` for the BackChannel impl. Transcribe `send_release` and `recv_release_nonblocking` into the Linux variant. Use the existing UDS connection (no second socket — bidirectional reads/writes on the same UnixStream).

Note: this means `FdPassingConnection::Linux` is now bidirectional. UDS UnixStream is naturally bidirectional, but the publisher accept-thread today only writes. Add a per-subscriber polling read in send_with_fd (drain pending acks before next send) OR add a separate `recv_release_ack` method that reads from one connected subscriber stream. Pick option (b) — KISS, no thread changes."

- [ ] **Step 2: Test back-channel roundtrip**

`iceoryx2-cal/tests/zero_copy_connection_dmabuf_tests.rs::back_channel_release_roundtrip`:
1. Open publisher + subscriber Linux instances.
2. Publisher `send_with_fd` carrying token (use `send_with_meta_and_fd` if Meta delivery is wired up).
3. Subscriber `recv_with_fd` returns `(meta, fd, len)`.
4. Subscriber calls connection's `send_release_ack(token)`.
5. Publisher calls connection's `recv_release_ack()` and asserts `Some(token)`.

- [ ] **Step 3: Add publish_with_token / receive_with_token in iceoryx2 port types**

Brief mos:rust-coder: "In `iceoryx2/src/port/dmabuf_publisher.rs`, add `pub fn publish_with_token(&mut self, meta: Meta, fd: BorrowedFd, len: u64, token: u64) -> Result<(), PublishError>`. The token travels in the wire's `meta_size` slot (we'll widen wire to `[8B len][8B token][8B meta_size][var Meta][fd]` — bump wire-format version constant). Symmetric `receive_with_token()` in subscriber returning `(Meta, OwnedFd, u64 token, u64 len)`. Existing `publish` / `receive` methods auto-allocate token from internal counter (preserves current behavior)."

- [ ] **Step 4: Add release(token) to subscriber + recv_release_ack to publisher**

Brief mos:rust-coder: "Add `pub fn release(&mut self, token: u64) -> Result<(), ReleaseError>` on DmaBufServiceSubscriber, forwarding to `connection.send_release_ack(token)`. Add `pub fn recv_release_ack(&mut self) -> Result<Option<u64>, ReleaseError>` on DmaBufServicePublisher, forwarding to `connection.recv_release_ack()`. New `enum ReleaseError { ConnectionFailed }`."

- [ ] **Step 5: Migrate Cargo workspace dependency wiring**

Brief mos:rust-coder: "Per the recent commit `2e98e0cb5 build(workspace): route iceoryx2-dmabuf via workspace.dependencies path`, the new branch must also route iceoryx2-dmabuf via workspace.dependencies. Update `Cargo.toml` (workspace root) accordingly so external consumers see the path-routed crate during the PR review."

- [ ] **Step 6: Run + commit**

```bash
cargo test -p iceoryx2-cal --test zero_copy_connection_dmabuf_tests
cargo test -p iceoryx2 --test service_dmabuf_publish_subscribe_tests
cargo clippy --workspace --all-features --all-targets -- -D warnings
git add -A
git commit -m "feat(iceoryx2): add back-channel + token API to dmabuf::Service ports

DmaBufServicePublisher::recv_release_ack and DmaBufServiceSubscriber::release
provide the consumer to producer pool-ack back-channel for mos-frame
buffer-refcount integration.

publish_with_token / receive_with_token expose the wire token directly
so an external AckLedger can own token assignment.

Wire frame v2: [8B len][8B token][8B meta_size][var Meta][SCM_RIGHTS fd].

Refs #1570"
```

- [ ] **Step 7: Update spec + arch documents to reflect v2 wire**

Brief mos:docs-writer: "Update `iceoryx2-dmabuf/specs/spec-dmabuf-service-variant.adoc` requirement §17 to describe the v2 wire format `[8B len][8B token][8B meta_size][var Meta][fd]` and add new requirements for `release` / `recv_release_ack`. Update `arch-dmabuf-service-variant.adoc` D3 (wire) and D4 (token correlation) sections to note that token IS surfaced in API for ack semantics — clarify it's NOT in iceoryx2 user-header (still maintained), it travels in our cal-layer wire frame."

- [ ] **Step 8: Verify upstream #1551 prealloc compatibility**

Brief mos:tester: "Add an integration test `iceoryx2/tests/service_dmabuf_publish_subscribe_tests.rs::dmabuf_service_supports_override_max_preallocated`. Use the upstream `#1551` `override_max_preallocated_samples` API on the dmabuf publisher builder (if applicable to fd-passing — may be N/A since we don't preallocate fds). If N/A, document why in a comment. Do NOT regress the prealloc API for non-dmabuf services."

---

## Task 6: Shrink iceoryx2-dmabuf to typed convenience (TDD)

**Files:**
- Delete sidecar source files (lib + bin + tests).
- Modify `iceoryx2-dmabuf/src/lib.rs`, `dmabuf_publisher.rs`, `dmabuf_subscriber.rs`, `Cargo.toml`.
- Create `iceoryx2-dmabuf/tests/it_roundtrip.rs`.

- [ ] **Step 1: Delete sidecar source files**

```bash
git rm iceoryx2-dmabuf/src/back_channel.rs \
       iceoryx2-dmabuf/src/error.rs \
       iceoryx2-dmabuf/src/path.rs \
       iceoryx2-dmabuf/src/publisher.rs \
       iceoryx2-dmabuf/src/scm.rs \
       iceoryx2-dmabuf/src/side_channel.rs \
       iceoryx2-dmabuf/src/subscriber.rs \
       iceoryx2-dmabuf/src/token.rs \
       iceoryx2-dmabuf/src/wire.rs
git rm iceoryx2-dmabuf/src/bin/fd_sidecar_crash_pub.rs \
       iceoryx2-dmabuf/src/bin/fd_sidecar_fd_identity.rs
git rm iceoryx2-dmabuf/tests/back_channel.rs \
       iceoryx2-dmabuf/tests/dmabuf_roundtrip.rs \
       iceoryx2-dmabuf/tests/error_paths.rs \
       iceoryx2-dmabuf/tests/it_crash_midsend.rs \
       iceoryx2-dmabuf/tests/it_fanout.rs \
       iceoryx2-dmabuf/tests/it_fd_identity.rs \
       iceoryx2-dmabuf/tests/it_service_gone.rs \
       iceoryx2-dmabuf/tests/it_socket_gone.rs \
       iceoryx2-dmabuf/tests/peercred_mismatch.rs \
       iceoryx2-dmabuf/tests/prop_roundtrip.rs \
       iceoryx2-dmabuf/tests/refcount_survival.rs \
       iceoryx2-dmabuf/tests/unit_dmabuf_publisher.rs \
       iceoryx2-dmabuf/tests/unit_generic_service.rs \
       iceoryx2-dmabuf/tests/unit_path.rs \
       iceoryx2-dmabuf/tests/unit_scm.rs \
       iceoryx2-dmabuf/tests/unit_token.rs
```

- [ ] **Step 2: Rewrite lib.rs**

`iceoryx2-dmabuf/src/lib.rs`:
- `#![cfg(target_os = "linux")]`
- `#![cfg(feature = "dma-buf")]`
- Doc comment.
- `pub mod dmabuf_publisher;` `pub mod dmabuf_subscriber;`
- `pub use dmabuf_publisher::DmaBufPublisher;` `pub use dmabuf_subscriber::DmaBufSubscriber;`
- `pub use dma_buf::{DmaBuf, MappedDmaBuf};`
- `pub use iceoryx2::service::dmabuf::Service as DmaBufService;`

- [ ] **Step 3: Rewrite dmabuf_publisher.rs**

`DmaBufPublisher<Meta>` newtype:
- Holds `_node: Node<Service>` and `inner: DmaBufServicePublisher<Service, Meta>`.
- `pub fn create(service_name: &str) -> Result<Self, Error>`:
  - `NodeBuilder::new().create::<Service>()` mapped to `Error::NodeCreate`.
  - Convert `service_name` to `ServiceName` via `try_into`, mapped to `Error::Service`.
  - `node.service_builder(&svc_name).publish_subscribe::<Meta>().open_or_create()` mapped to `Error::Service`.
  - `factory.publisher_builder().create()` mapped to `Error::Publisher`.
- `pub fn publish(&mut self, meta: Meta, buf: &dma_buf::DmaBuf) -> Result<(), Error>`:
  - `let fd = buf.as_fd().try_clone_to_owned().map_err(Error::FdDup)?;`
  - `let len = buf.size() as u64;`
  - `self.inner.publish(meta, fd.as_fd(), len).map_err(...)`.
- `pub enum Error { NodeCreate(String), Service(String), Publisher(String), Send(String), FdDup(io::Error) }`.

- [ ] **Step 4: Rewrite dmabuf_subscriber.rs symmetrically**

`DmaBufSubscriber<Meta>` newtype:
- Symmetric `create` building subscriber from factory.
- `pub fn receive(&mut self) -> Result<Option<(Meta, dma_buf::DmaBuf)>, Error>`:
  - `self.inner.receive()?` returns `Option<(Meta, OwnedFd, u64)>`.
  - On `Some((meta, fd, _len))`, return `Some((meta, dma_buf::DmaBuf::from(fd)))`.

- [ ] **Step 5: Rewrite Cargo.toml**

Drop `sha1_smol`, `iceoryx2-pal-concurrency-sync`, `[target.'cfg(target_os = "linux")'.dependencies] rustix / libc / tracing`. Drop features `memfd`, `peercred`, `test-utils`. Keep `default = ["std"]`, `std = ["iceoryx2/std"]`, `dma-buf = ["dep:dma-buf"]`. Keep `iceoryx2 = { workspace = true }` and Linux-only `dma-buf = { version = "0.5", optional = true }`. Dev-deps: `libc`, `dma-heap` (Linux). Examples: `dmabuf-service-publisher` and `dmabuf-service-subscriber`. Tests: `it_roundtrip`, `it_dmabuf_identity`, `it_dmabuf_heap`, all `required-features = ["dma-buf"]`.

- [ ] **Step 6: Add it_roundtrip.rs**

Test:
1. `DmaBufPublisher::<u64>::create("test-roundtrip")`.
2. `DmaBufSubscriber::<u64>::create("test-roundtrip")`.
3. Sleep 20ms.
4. memfd_create + ftruncate(4096) → OwnedFd → DmaBuf::from(fd).
5. `pubr.publish(42, &buf)`.
6. Sleep 20ms.
7. `subr.receive()` returns `Some((42, _buf))`.

Gated `#![cfg(all(target_os = "linux", feature = "dma-buf"))]`.

- [ ] **Step 7: Run, fmt, clippy**

```bash
cargo fmt -p iceoryx2-dmabuf --check
cargo test -p iceoryx2-dmabuf --features dma-buf
cargo clippy -p iceoryx2-dmabuf --all-features --all-targets -- -D warnings
```

Expected: all green.

- [ ] **Step 8: Commit**

```bash
git add -u
git add iceoryx2-dmabuf/src/lib.rs \
        iceoryx2-dmabuf/src/dmabuf_publisher.rs \
        iceoryx2-dmabuf/src/dmabuf_subscriber.rs \
        iceoryx2-dmabuf/Cargo.toml \
        iceoryx2-dmabuf/tests/it_roundtrip.rs
git commit -m "refactor(iceoryx2-dmabuf): shrink to typed convenience over dmabuf::Service"
```

---

## Task 7: Migrate fd-identity + heap roundtrip tests

**Files:**
- Create: `iceoryx2-dmabuf/tests/it_dmabuf_identity.rs`
- Create: `iceoryx2-dmabuf/tests/it_dmabuf_heap.rs`

- [ ] **Step 1: Write it_dmabuf_identity.rs**

Reference: `git show feat/dmabuf-sidecar-git-consumable:iceoryx2-dmabuf/tests/it_dmabuf_fd_identity.rs`. Adapt:
- Use `DmaBufPublisher` / `DmaBufSubscriber` from new crate.
- Two-thread (or two-process) fstat assertion: publisher fstats its fd, subscriber fstats its received fd, assert `st_ino` equal.

- [ ] **Step 2: Run — must pass on Linux**

```bash
cargo test -p iceoryx2-dmabuf --features dma-buf --test it_dmabuf_identity
```

- [ ] **Step 3: Write it_dmabuf_heap.rs**

Reference: `git show feat/dmabuf-sidecar-git-consumable:iceoryx2-dmabuf/tests/it_dmabuf_heap_roundtrip.rs`. Skips when `/dev/dma_heap/system` is absent (`println!("skip: ...")` then `return`). Allocates 4096 bytes via `dma_heap::Heap::new(HeapKind::System).allocate(4096)`. Wraps as `dma_buf::DmaBuf`. Publishes, receives, calls `MappedDmaBuf::read` with a closure that asserts a known byte pattern (use `mapped.write` first to seed the pattern).

- [ ] **Step 4: Run + commit**

```bash
cargo test -p iceoryx2-dmabuf --features dma-buf
git add iceoryx2-dmabuf/tests/it_dmabuf_identity.rs \
        iceoryx2-dmabuf/tests/it_dmabuf_heap.rs
git commit -m "test(iceoryx2-dmabuf): migrate fd-identity + heap-roundtrip tests"
```

---

## Task 8: Examples + README

**Files:**
- Create: `iceoryx2-dmabuf/examples/publish_subscribe_dmabuf_service/{publisher,subscriber}.rs`
- Modify: `iceoryx2-dmabuf/README.md`

- [ ] **Step 1: Write publisher example**

Allocates from `/dev/dma_heap/system`, publishes 100 frames at ~30 fps over `dmabuf::Service`. Returns `Result<(), Box<dyn Error>>` so the body can use `?`. Uses `iceoryx2_dmabuf::DmaBufPublisher::<u64>::create("camera/frames")?` and a `for frame_id in 0u64..100` loop.

- [ ] **Step 2: Write subscriber example**

Subscribes to `"camera/frames"`. Loops on `subr.receive()`. On `Some((frame_id, buf))`, calls `buf.memory_map()?` then `mapped.read(...)` with a closure printing `frame_id` and `data[0]`. On `None`, sleeps 5ms.

- [ ] **Step 3: Rewrite README.md**

Three sections:
1. Overview — `dmabuf::Service` variant and its position in iceoryx2's Service family.
2. Quick start — example pair from above.
3. Migration from sidecar — table mapping old types (`FdSidecarPublisher`, etc.) to new (`DmaBufPublisher` over `dmabuf::Service`).

- [ ] **Step 4: Build examples + commit**

```bash
cargo build -p iceoryx2-dmabuf --features dma-buf --examples
git add iceoryx2-dmabuf/examples/publish_subscribe_dmabuf_service/ \
        iceoryx2-dmabuf/README.md
git commit -m "docs(iceoryx2-dmabuf): replace examples + README for service variant"
```

---

## Task 9: Migrate benchmarks

**Files:**
- Modify: `benchmarks/dmabuf/src/{bench_latency,bench_throughput,bench_fanout,main}.rs`
- Modify: `benchmarks/dmabuf/Cargo.toml`

- [ ] **Step 1: Replace FdSidecar* with DmaBufService* (or DmaBufPublisher convenience)**

For each `bench_*.rs`, replace publisher/subscriber types and send/recv signatures. Keep metric collection logic (p50/p95/p99 buckets, frame counts).

- [ ] **Step 2: Run all three benchmarks on Linux**

```bash
cargo run -p iceoryx2-benchmarks-dmabuf --release -- latency
cargo run -p iceoryx2-benchmarks-dmabuf --release -- throughput
cargo run -p iceoryx2-benchmarks-dmabuf --release -- fanout
```

Expected: comparable numbers to sidecar prototype. Note any regression in `benchmarks/dmabuf/RESULTS.md`.

- [ ] **Step 3: Commit**

```bash
git add benchmarks/dmabuf/
git commit -m "chore(benchmarks): migrate dmabuf benchmarks to dmabuf::Service"
```

---

## Task 10: Mark old specs as superseded + final checks

**Files:**
- Modify: `iceoryx2-dmabuf/specs/arch-fd-sidecar.adoc:6` (status line)
- Modify: `iceoryx2-dmabuf/specs/spec-dmabuf-typed-transport.adoc:7` (status line)
- Modify: `doc/release-notes/iceoryx2-unreleased.md` — Breaking + Features sections
- Workspace: full clippy/test/fmt sweep

- [ ] **Step 1: Add supersession header to arch-fd-sidecar.adoc**

Replace status line (line 6) with:

`Status:: Superseded by Design C (arch-dmabuf-service-variant.adoc). Retained for historical context and primitive reference.`

- [ ] **Step 2: Same on spec-dmabuf-typed-transport.adoc** (line 7)

- [ ] **Step 3: Update release notes**

In `doc/release-notes/iceoryx2-unreleased.md`, replace existing iceoryx2-dmabuf bullets with the new `dmabuf::Service` description. Under Breaking, list:
- `iceoryx2-cal::shm_allocator::ShmAllocationError` gains `ExternallyAllocated`.
- `iceoryx2-dmabuf` API redesigned around `dmabuf::Service`; sidecar-era types removed.

- [ ] **Step 4: Run full workspace sweep**

```bash
cargo fmt --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo test --workspace --features dma-buf
cargo doc --workspace --no-deps --all-features 2>&1 | grep -i "broken\|warning"
cargo build --workspace --no-default-features --target aarch64-apple-darwin
```

Expected: every command green, doc warnings empty, darwin build clean.

- [ ] **Step 5: Commit + final verification**

```bash
git add iceoryx2-dmabuf/specs/arch-fd-sidecar.adoc \
        iceoryx2-dmabuf/specs/spec-dmabuf-typed-transport.adoc \
        doc/release-notes/iceoryx2-unreleased.md
git commit -m "docs: mark sidecar specs superseded; update release notes for dmabuf::Service"

grep -r "FdSidecar\|BackChannel\|BufferReleased" iceoryx2 iceoryx2-cal iceoryx2-dmabuf benchmarks/dmabuf 2>/dev/null
```

Expected last command output: empty.

---

## Task 11: PR draft

**Files:**
- Create: `iceoryx2-dmabuf/docs/pr/PR-MESSAGE-design-c.md`

- [ ] **Step 1: Write PR-MESSAGE-design-c.md**

Sections:
- Title: `[#1570] Add dmabuf::Service variant + iceoryx2-cal fd-backed concepts`
- Summary (2-3 paragraphs explaining the reshape from sidecar)
- Reference to maintainer comment on PR #1572
- Cargo features table
- Commit structure (per spec §49)
- Test evidence (Linux + Darwin)
- Migration guide for sidecar users
- Out of scope (multi-planar, async, sync_file, etc.)
- Questions for maintainers

- [ ] **Step 2: Open the PR (manual user action)**

User pushes the branch and opens the PR via `gh pr create`, referencing the maintainer's comment on PR #1572.

```bash
git push -u origin feat/dmabuf-service-variant
gh pr create --title "[#1570] Add dmabuf::Service variant + iceoryx2-cal fd-backed concepts" \
             --body-file iceoryx2-dmabuf/docs/pr/PR-MESSAGE-design-c.md
```

- [ ] **Step 3: Commit PR draft + close out**

```bash
git add iceoryx2-dmabuf/docs/pr/PR-MESSAGE-design-c.md
git commit -m "docs: PR message draft for dmabuf::Service variant"
git push origin feat/dmabuf-service-variant
```

---

## Self-review notes

**Spec coverage.** Every numbered requirement in `spec-dmabuf-service-variant.adoc` maps to a task. Requirements 1-2 → Task 0; 3-7 → Task 1; 8-14 → Task 2; 15-22 → Task 3; 23-26 → Task 4; 27-31 → Task 5; 32-38 → Task 6; 39-40 → Tasks 6+7; 41 → Task 9; 42-48 → Task 10; 49 → distributed across all task commits.

**Type consistency.** `DmaBufServicePublisher` / `DmaBufServiceSubscriber` (in `iceoryx2`), `DmaBufPublisher` / `DmaBufSubscriber` (in `iceoryx2-dmabuf`) — distinct names, distinct crates, no collision.

**Risk areas.**
1. *Task 5 plumbing.* `Publisher<S, Meta, ()>` connection access requires touching `iceoryx2/src/port/publisher.rs` to expose `Arc<S::Connection>`. If the existing publisher cannot be modified (stability), fall back to a fully bespoke `dmabuf::Service`-only port that bypasses the generic publisher. Document the choice in the commit message.
2. *Task 4 Service trait associated types.* `ResizableSharedMemory` may not have a sensible `dmabuf::Linux<NoAllocator>` shape. Likely needs an additional `Resizable` newtype wrapper in `shared_memory::dmabuf` (Task 2 may need a sub-task).
3. *Task 1 ShmAllocatorConfig: Copy bound.* If `OwnedFd: !Copy` blocks the trait, the trait widens (drop Copy from base) — surface as a Task 1 sub-decision.
4. *Tasks 2 + 3 NamedConcept plumbing.* fd-backed SHM and SCM_RIGHTS connection don't fit cleanly under `NamedConcept` (which is path-based). Either implement minimal stubs returning the UDS path as the "name", or relax the bound on `dmabuf::Service::SharedMemory` / `Connection`. Resolve when the trait surface fights compilation.

These risks are tracked as inline notes within the relevant task steps; surface them to the user before continuing if encountered during execution.

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-05-05-dmabuf-service-variant.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — Dispatch fresh subagent per task, review between tasks, fast iteration. Risks above mean tasks 1-5 will probably need user check-in between subagents.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

**Which approach?**
