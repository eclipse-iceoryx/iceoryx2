# Roadmap

## Main Focus

The list is sorted by priority

### Planned For v0.7

* [x] Python Language Bindings
* [x] Basic Host2Host Communication based on zenoh
* [ ] `iox2` cli service debugging
* [ ] Detailed getting started documentation for all supported languages

### After v0.7

* [ ] Embedded Platforms: QNX 7.1 & QNX 8.0
* [ ] Blackboard Messaging Pattern

## Backlog

> [!IMPORTANT]
>
> The features in our backlog have no set timeline and will be eventually worked
> on if capacity becomes available.
>
> If there is a feature you would like to see completed within a certain
> timeline,
> [please don't hesitate to reach out!](https://github.com/eclipse-iceoryx/iceoryx2?tab=readme-ov-file#commercial-support)
> Features in the backlog (or new ones) can be sponsored to be prioritized.
>
> The funds we receive through sponsored features help us keep the lights
> on - they go towards making `iceoryx2` awesome and
> making sure we can continue maintaining it for everyone.

### Moonshots

* [ ] high performance d-bus alternative (thanks to true zero-copy)
* [ ] `#![no_std]` on `nightly` on all tier 1 platforms
* [ ] `#![no_std]` on `stable` on all tier 1 platforms
* [x] completely dynamic setup with dynamic shared memory
* [ ] iceoryx2 on a rover on the moon
* [ ] iceoryx2 can communicate with every SOA protocol transparently with
      gateways that are activated automatically

### Shared Memory Container & Types

* [ ] Make `iceoryx2_bb_container` public with announcement
* [ ] Create and document dynamic size container concept for shared memory and
      apply it to all existing containers: `ByteString`, `Vec`, `Queue`
    * Open Question: How can these containers be cloned, copied?
* [ ] Introduce additional containers: `HashMap`, `Tree`, `Set`, `List`
* [ ] Introduce elementary types, look into: `simple-si-units` crate
    * Add types like: memory size, percentage, strict percentage (0..100), data
    throughput, resolution (further types found in informatics)
* [x] Add `derive` proc macro to ensure that only shm compatible types can be
      transferred via zero-copy

### Language Bindings

* [x] C
* [x] C++
* [x] Python
* [ ] C#
* [ ] Dash/Flutter
* [ ] Kotlin
* [ ] Go
* [ ] Lua
* [ ] Swift
* [ ] Typescript
* [ ] Zig

### Building Blocks

* [x] WaitSet - event multiplexer based on reactor pattern
* [x] Introduce trait and proc macro to generate types that can be sent via
      shared memory (`ZeroCopySend`)
    * ensure that only these types are used for inter-process communication

### Gateways

* [ ] Host2Host Communication based on <https://github.com/smoltcp-rs/smoltcp>
* [ ] Host2Host Communication based on zenoh
* [ ] mqtt (rumqtt)
* [ ] dds (rustdds or dustdds)
* [ ] zenoh
* [ ] someip (maybe sommr)
* [ ] dbus (zbus)
* [ ] websocket

### Microservices (Quality of Life Improvements)

#### iceoryx Tooling

* [x] Service Discovery
* [ ] Introspection Service
* [ ] Process Monitor (process can register and cleans up resources when process
      dies)
* [ ] Health Monitor
* [x] Basic command line introspection tooling
* [ ] Tooling for advanced introspection, cool WebGUI
* [ ] Command line client as interface to microservices
* [ ] `iox2` cli service debugging

#### Tools and Gadgets

* [ ] System Monitor (show CPU load etc. like top)

### Communication

* [x] publish subscribe
* [x] events
* [ ] integrated serialization to send non-shm compatible types, see:
      <https://github.com/rkyv/rkyv>
* [ ] Single Publisher Subscribe with history
* [ ] Multi Publisher without history (except there is a brilliant idea on how
      to realize it with history)
* [x] Request Response Messaging Pattern
* [ ] Blackboard Messaging Pattern
* [ ] Log messaging pattern
* [ ] Pipeline Messaging Pattern
* [ ] PubSub, ReqRes, Pipeline variant that works with copies (poor mans mixed
      criticality)
* [ ] Zero-copy GPU communication with Cuda, NvSci, Vulkan
* [ ] Zero-copy across hypervisor partitions
* [ ] Zero-copy via QEMU ivshmem:
      <https://www.qemu.org/docs/master/system/devices/ivshmem.html>
* [ ] dmabuf support, see:
      <https://blaztinn.gitlab.io/post/dmabuf-texture-sharing/>
* [ ] Support dynamic sized types in a memory efficient manner
    * Buddy allocator for sender data-segment
    * Introduce runtime fixed-size types
* [x] Untyped API
* [ ] Zero-copy based on dma-heap in linux

### Expert/Advanced Features

* [ ] Filtering/Routing of messages in pub-sub
* [ ] Handle approach to resend samples that could not be delivered caused by a
      full queue in pub-sub

### Robustness

* [X] Node as basis for monitoring and resource cleanup
* [ ] Add ability to recover samples when subscriber died
    * add sample tracker into ZeroCopyConnection
    * add detection when subscriber returns non-received samples
* [ ] Large Communication Setup Support
    * handle the restriction of a max amount of POSIX shared memory objects of an
    OS
    * add `iceoryx2_cal` implementations that are using the `SharedMemoryGroup`

### Platform Support

* [ ] Android
* [ ] Android Automotive
* [x] Linux
* [x] Windows
* [x] Mac Os
* [ ] iOS
* [ ] WatchOS
* [x] FreeBSD
* [ ] FreeRTOS
* [ ] QNX
* [ ] VxWorks
* [ ] BareMetal
* [ ] Sandbox Mode (only process internal communication)

### Hardware Support

* [x] x86_64
* [x] aarch64
* [ ] armv7
* [ ] x32
* [ ] risc-v

### Framework Integration

* [ ] ROS2 rmw binding
* [ ] dora-rs integration

### Safety & Security

* [ ] Mixed Criticality setup, e.g. applications do not interfere with each
      other
* [x] Sample Tracking when application crashes
* [ ] Identity and Access Management, e.g. service that create additional
      services
* [ ] Use Kani, Loom and Miri for tests of lock-free constructs

### Development

* [ ] Tracing integration for advanced performance measurements, see google
      chrome chrome://tracing/ and flame graphs See lttng, add trace points to
      the source code on the important functions

### Quality Of Life Improvements

* [ ] Evaluate and refactor basic error handling approach based on enums
    * all error classes should implement `core::error::Error` for the non `no_std`
    build (maybe use `thiserror`)
    * should be compatible with `anyhow` and `eyre`
    * explore error pyramid concept
* [ ] Rename `enable_safe_overflow` into `set_safe_overflow` in `ServiceBuilder`
      `publish_subscribe`
    * or maybe rename it into behavior: queue and ringbuffer, get inspired by
    crossbeam queues
* [x] Provide `[T]` (slice) as special transmission type for pub/sub
    * `loan_slice`, `loan_uninit_slice` and `loan_uninit_slice_with_alignment`
* [ ] QoS feature for blocking publisher or pub/sub failures to perform custom
      error handling or expert behavior
    * explore implementation as trait
    * explore implementation as callback
* [x] Explore if it is useful to have the same service name for different
      messaging patterns
    * separate them via internal suffix/prefix
    * simple use case: pub/sub + event to notify subscriber to notify sample send
    * would reduce error handling: connect to service with wrong messaging pattern
* [x] Implement Resizable SharedMemoryConcept that is able to extend the shared
      memory by adding additional POSIX shared memory objects

### Integration Into Other Projects

* [ ] Maybe Hyprland
