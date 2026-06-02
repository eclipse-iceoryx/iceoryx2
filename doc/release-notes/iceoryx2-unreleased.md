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
* [#1584](https://github.com/eclipse-iceoryx/iceoryx2/issues/1584) Introduce `Node::force_remove_service` to remove corrupted services manually.
* [#1544](https://github.com/eclipse-iceoryx/iceoryx2/issues/1544) Announce service removal over the tunnel to remote hosts
* [#1616](https://github.com/eclipse-iceoryx/iceoryx2/issues/1616) Add reactive execution mode to tunnel

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1673](https://github.com/eclipse-iceoryx/iceoryx2/issues/1673) Thread-stack-size is the same as process-stack-size on all platforms.
* [#1690](https://github.com/eclipse-iceoryx/iceoryx2/issues/1690) Fix dependencies in iceoryx2-bb-loggers to re-enable `bazel query`.
* [#1695](https://github.com/eclipse-iceoryx/iceoryx2/issues/1695) Remove port_tag when stale resources of port are removed.
* [#1699](https://github.com/eclipse-iceoryx/iceoryx2/issues/1699) Enforce synchronization with compare exchange in UnrestrictedAtomic, RobustUniqueIndexSet and Container.

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#996](https://github.com/eclipse-iceoryx/iceoryx2/issues/996) Move BumpAllocator from iceoryx2-bb-memory into iceoryx2-bb-elementary

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1610](https://github.com/eclipse-iceoryx/iceoryx2/issues/1610) Add `no_std` tests for tunnel

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

<!-- markdownlint-enable MD013 -->
