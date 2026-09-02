<!-- markdownlint-disable MD013 The new format requires longer lines -->

# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#820](https://github.com/eclipse-iceoryx/iceoryx2/issues/820) Allow restricting the gateway to a configurable allowlist of services
* [#925](https://github.com/eclipse-iceoryx/iceoryx2/issues/925) Adjust event API and guarantee that events can be always delivered.
* [#1185](https://github.com/eclipse-iceoryx/iceoryx2/issues/1185) Make history configurable per subscriber
* [#1584](https://github.com/eclipse-iceoryx/iceoryx2/issues/1584) Introduce `Node::force_remove_service` to remove corrupted services manually.
* [#1544](https://github.com/eclipse-iceoryx/iceoryx2/issues/1544) Announce service removal over the gateway to remote hosts
* [#1616](https://github.com/eclipse-iceoryx/iceoryx2/issues/1616) Add reactive execution mode to gateway
* [#1649](https://github.com/eclipse-iceoryx/iceoryx2/issues/1649) Add `IOX2_DEFINE_TYPE_NAME` to the C++ bindings to set the cross-language type name for types that cannot carry an `IOX2_TYPE_NAME` member
* [#1707](https://github.com/eclipse-iceoryx/iceoryx2/issues/1707) Expose `CustomHeaderMarker` and `CustomPayloadMarker` in C++ bindings
* [#1722](https://github.com/eclipse-iceoryx/iceoryx2/issues/1722) Remove allocations in gateway hot path
* [#1742](https://github.com/eclipse-iceoryx/iceoryx2/issues/1742) Add (work-in-progress) gateway implementation for ROS 2
* [#1745](https://github.com/eclipse-iceoryx/iceoryx2/issues/1745) Add Flatbuffers support for publish-subscribe and request-response payloads
* [#1773](https://github.com/eclipse-iceoryx/iceoryx2/issues/1773) Make ports identifiable by name
* [#1798](https://github.com/eclipse-iceoryx/iceoryx2/issues/1798) Add support for musl 1.2.x
* [#1813](https://github.com/eclipse-iceoryx/iceoryx2/issues/1813) Add API to deliver events to specific listener only
* [#1958](https://github.com/eclipse-iceoryx/iceoryx2/issues/1958) Allow projects to acknowledge intentional Cargo builds from CMake

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#81](https://github.com/eclipse-iceoryx/iceoryx2/issues/81) Re-enable the `FetchableSignal::Continue` (`SIGCONT`) signal variant.
* [#156](https://github.com/eclipse-iceoryx/iceoryx2/issues/156) Remove `fchmod`/`shm_open` macOS workarounds; route permissions through a trampoline state file.
* [#588](https://github.com/eclipse-iceoryx/iceoryx2/issues/588) Replace deprecated `serde_yaml` dependency with `yaml_serde`.
* [#1152](https://github.com/eclipse-iceoryx/iceoryx2/issues/1152) Fix `no_std` build of the console logger on platforms other than linux and nto.
* [#1548](https://github.com/eclipse-iceoryx/iceoryx2/issues/1548) Fix Payload data lifetime tracking in python ffi by anchoring views to their owning Sample.
* [#1673](https://github.com/eclipse-iceoryx/iceoryx2/issues/1673) Thread-stack-size is the same as process-stack-size on all platforms.
* [#1695](https://github.com/eclipse-iceoryx/iceoryx2/issues/1695) Remove port_tag when stale resources of port are removed.
* [#1708](https://github.com/eclipse-iceoryx/iceoryx2/issues/1708) Remove `services` from gateway conformance test crate to fix a linker error on macOS.
* [#1718](https://github.com/eclipse-iceoryx/iceoryx2/issues/1718) Protect `ProcessState` from accidental file lock release.
* [#1737](https://github.com/eclipse-iceoryx/iceoryx2/issues/1737) Fix error log output in Windows for languages with non UTF-8 characters.
* [#1739](https://github.com/eclipse-iceoryx/iceoryx2/issues/1739) Make sure MSVC defines __cplusplus with accurate value
* [#1746](https://github.com/eclipse-iceoryx/iceoryx2/issues/1746) Disable `POSIX_SUPPORT_FILE_LOCK_FOR_SHARED_MEMORY` on FreeBSD and move CI job for FreeBSD to main pipeline
* [#1777](https://github.com/eclipse-iceoryx/iceoryx2/issues/1777) Fix service root folder creation named concept of iceoryx2-cal fixing execution on Windows platform.
* [#1786](https://github.com/eclipse-iceoryx/iceoryx2/issues/1786) Disable transport_compression feature in Zenoh.
* [#1788](https://github.com/eclipse-iceoryx/iceoryx2/issues/1788) Skip non-UTF-8 entries in `Node::list()` instead of panicking.
* [#1792](https://github.com/eclipse-iceoryx/iceoryx2/issues/1792) Set key eq comparison function in language bindings for blackboard opener.
* [#1797](https://github.com/eclipse-iceoryx/iceoryx2/issues/1797) Reclaim disconnected request-response client connections when fire-and-forget requests are disabled.
* [#1800](https://github.com/eclipse-iceoryx/iceoryx2/issues/1800) iceoryx2-cxx: CleanupState is defined in global namespace
* [#1807](https://github.com/eclipse-iceoryx/iceoryx2/issues/1807) Fix generated C FFI strings for `UPPER_SNAKE_CASE` enum variants.
* [#1810](https://github.com/eclipse-iceoryx/iceoryx2/issues/1810) Make mgmt segment globally accessible
* [#1844](https://github.com/eclipse-iceoryx/iceoryx2/issues/1844) Fix `semantic_string` macro for `rust-analyzer` auto-completion
* [#1868](https://github.com/eclipse-iceoryx/iceoryx2/issues/1868) Automatically create windows platform directories
* [#1872](https://github.com/eclipse-iceoryx/iceoryx2/issues/1872) Register FileDescriptor pyclass with the python module
* [#1878](https://github.com/eclipse-iceoryx/iceoryx2/issues/1878) Fix race in `pthread_create` on macOS and Windows
* [#1893](https://github.com/eclipse-iceoryx/iceoryx2/issues/1893) Fix C language `ipc` and `local` mapping for payload types
* [#1906](https://github.com/eclipse-iceoryx/iceoryx2/issues/1906) Do not set the exec bit for created resources
* [#1918](https://github.com/eclipse-iceoryx/iceoryx2/issues/1917) Fix wrong `CLOCK_MONOTONIC` constant on macOS (1 instead of 6) which broke `Time::now_with_clock(ClockType::Monotonic)`
* [#1924](https://github.com/eclipse-iceoryx/iceoryx2/issues/1924) Ensure discovery service node resources are removed during shutdown.
* [#1940](https://github.com/eclipse-iceoryx/iceoryx2/issues/1940) Fix private rustdoc link diagnostics.

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->
* [#996](https://github.com/eclipse-iceoryx/iceoryx2/issues/996) Move BumpAllocator from iceoryx2-bb-memory into iceoryx2-bb-elementary
* [#1326](https://github.com/eclipse-iceoryx/iceoryx2/issues/1326) Remove the Cargo.Bazel.lock from repository, it is still generated by Bazel but ignored in git
* [#1613](https://github.com/eclipse-iceoryx/iceoryx2/issues/1613) Remove `NonNullCompat` after moving to Rust 1.89
* [#1664](https://github.com/eclipse-iceoryx/iceoryx2/issues/1664) Evaluate service builder log origins lazily instead of formatting them on every call
* [#1776](https://github.com/eclipse-iceoryx/iceoryx2/issues/1776) Rename AtomicCopy::__for_each_field() to for_each_field()
* [#1845](https://github.com/eclipse-iceoryx/iceoryx2/issues/1845) Reduce imports for usage of the `semantic_string` macro
* [#1853](https://github.com/eclipse-iceoryx/iceoryx2/issues/1853) Improve error message in static asserts
* [#1891](https://github.com/eclipse-iceoryx/iceoryx2/issues/1891) Rename the tunnel to gateway and move its crates from `iceoryx2-services/` to a top-level `iceoryx2-gateway/` directory
* [#1928](https://github.com/eclipse-iceoryx/iceoryx2/issues/1928) Make Windows platform abstraction use the `libc` crate instead of `bindgen`
* [#1929](https://github.com/eclipse-iceoryx/iceoryx2/issues/1929) Make macOS platform abstraction use the `libc` crate instead of `bindgen`
* [#1930](https://github.com/eclipse-iceoryx/iceoryx2/issues/1930) Make FreeBSD platform abstraction use the `libc` crate instead of `bindgen`
* [#1931](https://github.com/eclipse-iceoryx/iceoryx2/issues/1931) Use ANSI escape sequences in iceoryx2-cli
* [#1942](https://github.com/eclipse-iceoryx/iceoryx2/issues/1942) Split implementation of gateway testing backend into modules
* [#1949](https://github.com/eclipse-iceoryx/iceoryx2/issues/1949) Take `&mut self` in gateway Discovery trait
* [#1955](https://github.com/eclipse-iceoryx/iceoryx2/issues/1955) Remove `as_mut_bytes` and `deref_mut` from the `String` API in iceoryx2-bb-container

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#3](https://github.com/eclipse-iceoryx/iceoryx2/issues/3) Use clang 21 in the CI
* [#1610](https://github.com/eclipse-iceoryx/iceoryx2/issues/1610) Add `no_std` tests for gateway
* [#1712](https://github.com/eclipse-iceoryx/iceoryx2/issues/1712) Add iceoryx2 version to static service config
* [#1714](https://github.com/eclipse-iceoryx/iceoryx2/issues/1714) Add locking for all file descriptor based constructs
* [#1815](https://github.com/eclipse-iceoryx/iceoryx2/issues/1815) Set Rust minimum required version (MSRV) to version 1.89.0
* [#1884](https://github.com/eclipse-iceoryx/iceoryx2/issues/1884) Bump `googletest` to 1.16.0
* [#1885](https://github.com/eclipse-iceoryx/iceoryx2/issues/1885) Bump bazel modules `bazel_features` to 1.32.0, `bazel_skylib` to 1.9.2, `platforms` to 1.1.0, `rules_cc` to 0.2.17, `rules_rust`/`rules_rust_bindgen` to 0.73.0 and `toolchains_llvm` to 1.8.0 with `llvm_version` 21.1.6
* [#1942](https://github.com/eclipse-iceoryx/iceoryx2/issues/1942) Reduce exeuction time of gateway backend tests

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

    let memory = [0u8; 8192];
    let sut = BumpAllocator::new(
        NonNull::<u8>::from_ref(&memory[0]),
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

1. The tunnel has been renamed to gateway. The crates were renamed and moved
   from `iceoryx2-services/` to a top-level `iceoryx2-gateway/` directory.

    | old                                          | new                                           |
    | -------------------------------------------- | --------------------------------------------- |
    | `iceoryx2-services-tunnel`                   | `iceoryx2-gateway`                            |
    | `iceoryx2-services-tunnel-backend`           | `iceoryx2-gateway-backend`                    |
    | `iceoryx2-services-tunnel-testing`           | `iceoryx2-gateway-testing`                    |
    | `iceoryx2-services-tunnel-conformance-tests` | `iceoryx2-gateway-conformance-tests`          |
    | `iceoryx2-integrations-zenoh-tunnel-backend` | `iceoryx2-integrations-zenoh-gateway-backend` |
    | `iceoryx2-integrations-zenoh-tunnel-cli`     | `iceoryx2-integrations-zenoh-gateway-cli`     |

    ```rust
    // old
    use iceoryx2_services_tunnel::{Tunnel, TunnelBuilder};

    let mut tunnel = Tunnel::<Service, Backend>::new().polled().create()?;
    let services = tunnel.tunneled_services();

    // new
    use iceoryx2_gateway::{Gateway, GatewayBuilder};

    let mut gateway = Gateway::<Service, Backend>::new().polled().create()?;
    let services = gateway.bridged_services();
    ```

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
        iceoryx2_gateway_conformance_tests::publish_subscribe_discovery,
        super::Ipc,
        super::TestBackend<super::Ipc>,
        super::Testing
    );
    ```

    The CLI was renamed accordingly. Backend binaries are discovered by the
    `iox2-gateway-` prefix, so a backend installed under the old
    `iox2-tunnel-` name is no longer found and must be reinstalled.

    ```console
    # old
    $ iox2 tunnel zenoh

    # new
    $ iox2 gateway zenoh
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

1. In iceoryx2-bb-container, methods `as_mut_bytes()` and `deref_mut()` are removed from the `String` API,
   This applies equally to `PolymorphicString`, `RelocatableString` and `StaticString`.

    ```rust
    use iceoryx2_bb_container::string::*;
    const CAPACITY: usize = 1234;
    let my_str = StaticString::<CAPACITY>::new();
    my_str.push_bytes(b"hello");
    
    let mut_str_slice = my_str.as_bytes(); // Compiler Error
    my_str.deref_mut()[0] = b'b'; // Compiler Error
    ```

<!-- markdownlint-enable MD013 -->
