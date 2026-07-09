<!-- markdownlint-disable MD013 The new format requires longer lines -->

# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#820](https://github.com/eclipse-iceoryx/iceoryx2/issues/820) Allow restricting the tunnel to a configurable allowlist of services via `Config::services` and the `--service`/`-s` flag on `iox2 tunnel zenoh`
* [#925](https://github.com/eclipse-iceoryx/iceoryx2/issues/925) Adjust event API and guarantee that events can be always delivered.
* [#1185](https://github.com/eclipse-iceoryx/iceoryx2/issues/1185) Make history configurable per subscriber
* [#1584](https://github.com/eclipse-iceoryx/iceoryx2/issues/1584) Introduce `Node::force_remove_service` to remove corrupted services manually.
* [#1544](https://github.com/eclipse-iceoryx/iceoryx2/issues/1544) Announce service removal over the tunnel to remote hosts
* [#1616](https://github.com/eclipse-iceoryx/iceoryx2/issues/1616) Add reactive execution mode to tunnel
* [#1649](https://github.com/eclipse-iceoryx/iceoryx2/issues/1649) Add `IOX2_DEFINE_TYPE_NAME` to the C++ bindings to set the cross-language type name for types that cannot carry an `IOX2_TYPE_NAME` member
* [#1707](https://github.com/eclipse-iceoryx/iceoryx2/issues/1707) Expose `CustomHeaderMarker` and `CustomPayloadMarker` in C++ bindings
* [#1722](https://github.com/eclipse-iceoryx/iceoryx2/issues/1722) Remove allocations in tunnel hot path
* [#1773](https://github.com/eclipse-iceoryx/iceoryx2/issues/1773) Make ports identifiable by name
* [#1798](https://github.com/eclipse-iceoryx/iceoryx2/issues/1798) Add musl support

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#156](https://github.com/eclipse-iceoryx/iceoryx2/issues/156) Remove `fchmod`/`shm_open` macOS workarounds; route permissions through a trampoline state file.
* [#588](https://github.com/eclipse-iceoryx/iceoryx2/issues/588) Replace deprecated `serde_yaml` dependency with `yaml_serde`.
* [#1548](https://github.com/eclipse-iceoryx/iceoryx2/issues/1548) Fix Payload data lifetime tracking in python ffi by anchoring views to their owning Sample.
* [#1673](https://github.com/eclipse-iceoryx/iceoryx2/issues/1673) Thread-stack-size is the same as process-stack-size on all platforms.
* [#1695](https://github.com/eclipse-iceoryx/iceoryx2/issues/1695) Remove port_tag when stale resources of port are removed.
* [#1708](https://github.com/eclipse-iceoryx/iceoryx2/issues/1708) Remove `services` from tunnel conformance test crate to fix a linker error on macOS.
* [#1718](https://github.com/eclipse-iceoryx/iceoryx2/issues/1718) Protect `ProcessState` from accidental file lock release.
* [#1739](https://github.com/eclipse-iceoryx/iceoryx2/issues/1739) Make sure MSVC defines __cplusplus with accurate value
* [#1746](https://github.com/eclipse-iceoryx/iceoryx2/issues/1746) Disable `POSIX_SUPPORT_FILE_LOCK_FOR_SHARED_MEMORY` on FreeBSD and move CI job for FreeBSD to main pipeline
* [#1777](https://github.com/eclipse-iceoryx/iceoryx2/issues/1777) Fix service root folder creation named concept of iceoryx2-cal fixing execution on Windows platform.
* [#1786](https://github.com/eclipse-iceoryx/iceoryx2/issues/1786) Disable transport_compression feature in Zenoh.
* [#1792](https://github.com/eclipse-iceoryx/iceoryx2/issues/1792) Set key eq comparison function in language bindings for blackboard opener.
* [#1800](https://github.com/eclipse-iceoryx/iceoryx2/issues/1800) iceoryx2-cxx: CleanupState is defined in global namespace
* [#1807](https://github.com/eclipse-iceoryx/iceoryx2/issues/1807) Fix generated C FFI strings for `UPPER_SNAKE_CASE` enum variants.

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#996](https://github.com/eclipse-iceoryx/iceoryx2/issues/996) Move BumpAllocator from iceoryx2-bb-memory into iceoryx2-bb-elementary
* [#1776](https://github.com/eclipse-iceoryx/iceoryx2/issues/1776) Rename AtomicCopy::__for_each_field() to for_each_field()

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1610](https://github.com/eclipse-iceoryx/iceoryx2/issues/1610) Add `no_std` tests for tunnel
* [#1712](https://github.com/eclipse-iceoryx/iceoryx2/issues/1712) Add iceoryx2 version to static service config
* [#1714](https://github.com/eclipse-iceoryx/iceoryx2/issues/1714) Add locking for all file descriptor based constructs

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1) Example text

### API Breaking Changes

1. The `Bumpallocator` from iceoryx2-bb-memory crate has been
   moved into the iceoryx2-bb-elementary crate and replaces it.
   The `Bumpallocator` is re-exported in iceoryx2-bb-memory and
   expects now a `NonNull<u8>` as start address and the size
   of the memory that the Allocator manages.

    ```rust
    // old
    use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;

    let memory = [0u8; 8192];
    let start_position: *mut u8 = memory.as_mut_ptr();
    let sut = BumpAllocator::new(start_position);

    // new

    use core::ptr::NonNull;

    use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
    use iceoryx2_bb_elementary_traits::non_null::NonNullCompat;

    let memory = [0u8; 8192];
    let sut = BumpAllocator::new(
        NonNull::<u8>::iox2_from_ref(&memory[0]),
        memory.len(),
    );
    ```

1. The `bump_allocator` module in the `iceoryx2-cal` package
 has been renamed to shm_bump_allocator.

    ```rust
    // old
    use iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator;

    // new
    use iceoryx2_cal::shm_allocator::shm_bump_allocator::BumpAllocator;
    ```

1. `Listener::{try|timed|blocking}_wait_one` has been removed and `Listener::{try|timed|blocking}_wail_all`
   has been renamed to `Listener::{try|timed|blocking}_wait`. The input argument has changed from `EventId`
   to `EventActivation`.

   ```rust
   // old: no more `**_wait_one()`
   while let Ok(Some(event_id)) = listener.timed_wait_one(CYCLE_TIME) {
       coutln!("event was triggered with id: {event_id:?}");
   }

   // old: renamed to `**_wait()`
   listener.timed_wait_all(|event_id| {
       coutln!("event was triggered with id: {event_id:?}");
   }, CYCLE_TIME)?;

   // new
   listener.timed_wait(|event| {
       // EventActivation provides access to the event.id and how often it was
       // notified with event.count.
       coutln!("event {:?} was notified {} times", event.id, event.count);
   }, CYCLE_TIME)?;
   ```

1. The tunnel conformance create name has been shortened.

    ```rust
    // old
    instantiate_conformance_tests_with_module!(
        ipc,
        iceoryx2_services_tunnel_conformance_tests::publish_subscribe_discovery,
        super::Ipc,
        super::TestBackend<super::Ipc>,
        super::Testing
    );

    // new
    instantiate_conformance_tests_with_module!(
        ipc,
        iceoryx2_tunnel_conformance_tests::publish_subscribe_discovery,
        super::Ipc,
        super::TestBackend<super::Ipc>,
        super::Testing
    );
    ```

1. `AtomicCopy::__for_each_field()` was renamed to `for_each_field()`.

    ```rust
    // old
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Foo {
        bar: u8,
        baz: u64,
    }
    
    unsafe impl AtomicCopy for Foo {
        fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
            // ...
        }
    }
    
    // new
    // ...
    unsafe impl AtomicCopy for Foo {
        fn for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
            // ...
        }
    }
    ```

1. In iceoryx2-cxx `CleanupState` was moved to the `iox2` namespace.

    ```c++
    // old
    CleanupState cleanup = node.try_cleanup_dead_nodes();

    // new
    iox2::CleanupState cleanup = node.try_cleanup_dead_nodes();
    ```

<!-- markdownlint-enable MD013 -->
