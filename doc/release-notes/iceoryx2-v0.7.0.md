# iceoryx2 v0.7.0

## [v0.7.0](https://github.com/eclipse-iceoryx/iceoryx2/tree/v0.7.0)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v0.6.1...v0.7.0)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Python language binding
  [#419](https://github.com/eclipse-iceoryx/iceoryx2/issues/419)
* Tunnel over zenoh for publish-subscribe and event services
  [#593](https://github.com/eclipse-iceoryx/iceoryx2/issues/593)
* Provide Official Yocto Recipe
  [#663](https://github.com/eclipse-iceoryx/iceoryx2/issues/663)
* All port factories implement `Send`
  [#768](https://github.com/eclipse-iceoryx/iceoryx2/issues/768)
* `RequestResponse` for entire current discovery state
  [#777](https://github.com/eclipse-iceoryx/iceoryx2/issues/777)
* `iox2 service listen` and `iox2 service notify`
  [#790](https://github.com/eclipse-iceoryx/iceoryx2/issues/790)
* Use minimal iceoryx_hoofs subset for iceoryx2 C++ bindings
  [#824](https://github.com/eclipse-iceoryx/iceoryx2/issues/824)
* PubSub ports implement `Send` + `Sync`, samples implement `Send` when using
  `**_threadsafe` service variant
  [#836](https://github.com/eclipse-iceoryx/iceoryx2/issues/836)
* ReqRes & events implement `Send` + `Sync` in
  `**_threadsafe` service variant
  [#838](https://github.com/eclipse-iceoryx/iceoryx2/issues/838)
* Platform support for QNX 7.1
  [#847](https://github.com/eclipse-iceoryx/iceoryx2/issues/847)
* Send/receive samples with `iox2` + simple record & replay
  [#884](https://github.com/eclipse-iceoryx/iceoryx2/issues/884)
* C example for service attributes
  [#909](https://github.com/eclipse-iceoryx/iceoryx2/issues/909)
* Example explaining the details of service types
  [#913](https://github.com/eclipse-iceoryx/iceoryx2/issues/913)
* CLI Record replay, service name is stored in record file
  [#929](https://github.com/eclipse-iceoryx/iceoryx2/issues/929)
* Flatmap `remove` API now returns `Option<T>`
  [#931](https://github.com/eclipse-iceoryx/iceoryx2/issues/931)
* Add script to list dependencies of multiple packages
  [#933](https://github.com/eclipse-iceoryx/iceoryx2/issues/933)
* Add a C++ Optional vocabulary type for use with C++ containers
  [#939](https://github.com/eclipse-iceoryx/iceoryx2/issues/939)
* Add graceful disconnect feature in client
  [#989](https://github.com/eclipse-iceoryx/iceoryx2/issues/989)
* Add custom mapping offset to shared memory POSIX wrapper
  [#1010](https://github.com/eclipse-iceoryx/iceoryx2/issues/1010)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Fix segmentation fault in event multiplexing C example
    [#766](https://github.com/eclipse-iceoryx/iceoryx2/issues/766)
* Fix lifetime in ThreadGuardedStackBuilder
    [#770](https://github.com/eclipse-iceoryx/iceoryx2/issues/770)
* Fix config file load failure
    [#772](https://github.com/eclipse-iceoryx/iceoryx2/issues/772)
* Make `Directory::create` thread-safe
    [#778](https://github.com/eclipse-iceoryx/iceoryx2/issues/778)
* Make CLI generated global config file accessible to all users
    [#786](https://github.com/eclipse-iceoryx/iceoryx2/issues/786)
* Make `iox2 config show` print the available options
    [#788](https://github.com/eclipse-iceoryx/iceoryx2/issues/788)
* Fix bug where CLI was not displaying positional arguments in usage help
    [#796](https://github.com/eclipse-iceoryx/iceoryx2/issues/796)
* Fix startup race in `pthread_create` on mac os platform
    [#799](https://github.com/eclipse-iceoryx/iceoryx2/issues/799)
* CMake no longer installs unusable binaries when FetchContent is used
    [#814](https://github.com/eclipse-iceoryx/iceoryx2/issues/814)
* Fix build and linker error on gcc 8 and 9
    [#855](https://github.com/eclipse-iceoryx/iceoryx2/issues/855)
* Miri complains about byte_string as_bytes* operations
    [#875](https://github.com/eclipse-iceoryx/iceoryx2/issues/875)
* Make changes to the config file backward compatible
    [#921](https://github.com/eclipse-iceoryx/iceoryx2/issues/921)
* Make `update_connections` public for all ports
    [#923](https://github.com/eclipse-iceoryx/iceoryx2/issues/923)
* Fix cleanup issue of stale dynamic configs
    [#927](https://github.com/eclipse-iceoryx/iceoryx2/issues/927)
* Fix memory leaks when port creation fails
    [#947](https://github.com/eclipse-iceoryx/iceoryx2/issues/947)
* Clean handling of thread CPU core affinity of `posix::Thread`
    [#962](https://github.com/eclipse-iceoryx/iceoryx2/issues/962)
* Set thread priority correctly, not always min value
    [#977](https://github.com/eclipse-iceoryx/iceoryx2/issues/977)
* Fix 32-bit data corruption issue in lock-free queues
    [#986](https://github.com/eclipse-iceoryx/iceoryx2/issues/986)
* Fix provenance in bump allocator and remove implicit Sync impl
    [#992](https://github.com/eclipse-iceoryx/iceoryx2/issues/992)
* Fix deadlock when system time changes and `wait` is called
    [#1000](https://github.com/eclipse-iceoryx/iceoryx2/issues/1000)
* Improve performance of the `fill` method of `FixedSizeVec`
    [#1006](https://github.com/eclipse-iceoryx/iceoryx2/issues/1006)
* Fix uninitialized user header in publisher, client, active request
    [#1014](https://github.com/eclipse-iceoryx/iceoryx2/issues/1014)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Implement `ZeroCopySend` for all system types
  [#732](https://github.com/eclipse-iceoryx/iceoryx2/issues/732)
* Remove trait re-exports from iceoryx2-bb-elementary
  [#757](https://github.com/eclipse-iceoryx/iceoryx2/issues/757)
* Make POSIX user- and group details optional
  [#780](https://github.com/eclipse-iceoryx/iceoryx2/issues/780)
* Add `recommended::Ipc` and `recommended::Local` to iceoryx2 concepts for
  to provide link the best implementation for the specific platform
  [#806](https://github.com/eclipse-iceoryx/iceoryx2/issues/806)
* Introduce newtypes for 'uid' and 'gid'
  [#822](https://github.com/eclipse-iceoryx/iceoryx2/issues/822)
* Make default max event ID smaller
  [#828](https://github.com/eclipse-iceoryx/iceoryx2/issues/828)
* Remove the `config/iceoryx2.toml` to reduce effort to keep the
  built in `iceoryx.toml` and `config.rs` in sync
  [#831](https://github.com/eclipse-iceoryx/iceoryx2/issues/831)
* Removed `MetaVec::is_initialized` field
  [#900](https://github.com/eclipse-iceoryx/iceoryx2/issues/900)
* Enable standalone build of the C and C++ bindings
  [#942](https://github.com/eclipse-iceoryx/iceoryx2/issues/942)
* Adjust visibility to `Service` constructs to allow customization
  [#954](https://github.com/eclipse-iceoryx/iceoryx2/issues/954)
* Move C and C++ language bindings to the top level directory
  [#963](https://github.com/eclipse-iceoryx/iceoryx2/issues/963)

### Testing

* Create E2E Tests for all examples
  [#730](https://github.com/eclipse-iceoryx/iceoryx2/issues/730)

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Added brief cross-compile documentation
  [#70](https://github.com/eclipse-iceoryx/iceoryx2/issues/70)
* Added development documentation for QNX
  [#847](https://github.com/eclipse-iceoryx/iceoryx2/issues/847)
* Set up automated documentation builds for C/C++/Python hosted on GitHub Pages
  [#920](https://github.com/eclipse-iceoryx/iceoryx2/issues/920)

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Add `NodeId` to sample header (to prevent loopback in tunnels)
  [#593](https://github.com/eclipse-iceoryx/iceoryx2/issues/593)
* Add API to prevent self notification `Notifier::__internal_notify()`
  [#794](https://github.com/eclipse-iceoryx/iceoryx2/issues/794)
* Enable the usage of semaphore based events in C/C++
  [#795](https://github.com/eclipse-iceoryx/iceoryx2/issues/795)
* Remove `impl Drop` restriction from `ZeroCopySend` trait
  [#908](https://github.com/eclipse-iceoryx/iceoryx2/issues/908)
* Introduce convenience `iceoryx2.hpp` header
  [#1016](https://github.com/eclipse-iceoryx/iceoryx2/issues/1016)

### Config Breaking Changes

1. The previously separate fields `root-path-unix` and `root-path-windows` have
  been unified into a single `root-path` entry in configs, located in
  the `[global]` section of `iceoryx2.toml`.

    The config file template from `config/iceoryx2.toml` was removed and
    please refer to `config/README.md` on how to generate a default config file.
  [#831](https://github.com/eclipse-iceoryx/iceoryx2/issues/831)

2. The default max event ID was reduced to 255 in order to have make bitset
   based event implementations work out of the box. If a larger event ID is
   required, it can either be changed in the `iceoryx2.toml` file or with the
   `event_id_max_value` in the event service builder.

    ```rust
    let event = node
        .service_builder(&"MyEventName".try_into()?)
        .event()
        .event_id_max_value(511)
        .open_or_create()?;
    ```

### API Breaking Changes

1. The `iceoryx2-ffi` crate is renamed to `iceoryx2-ffi-c` due to now also
   having a Python FFI package. The change should be transparent since the
   recommended way to use the C bindings is via the `iceoryx2-c` cmake package.

2. The custom `UserHeader`, `RequestHeader`, `ResponseHeader` must implement
   `Default`.
