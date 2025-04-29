# Roadmap

## v0.6

### Main Focus

The list is sorted by priority

* [x] Request/Response
* [ ] Zenoh Gateway
* [ ] Finalize C and C++ API (still some tiny things open)
* [x] Improve cross-compilation support

### Bonus

The list is sorted by priority

* [ ] Derive macro for SHM transferable types
* [ ] proof of concept with <https://github.com/rkyv/rkyv>
* [ ] `#![no_std]` for low-level crates
* [ ] Finish ROS 2 rmw binding (requires request/response)
* [ ] `serde`-based shm serialization to transmit arbitrary types

## Backlog

### Moonshots

* [ ] high performance d-bus alternative (thanks to true zero-copy)
* [ ] `#![no_std]` on `nightly` on all tier 1 platforms
* [ ] `#![no_std]` on `stable` on all tier 1 platforms
* [x] completely dynamic setup with dynamic shared memory
* [ ] iceoryx on a rover on the moon

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
* [ ] Python
* [ ] Swift
* [ ] Kotlin
* [ ] Typescript
* [ ] Lua
* [ ] Zig
* [ ] C#

### Building Blocks

* [x] WaitSet - event multiplexer based on reactor pattern
* [ ] Introduce trait and proc macro to generate types that can be sent via
      shared memory
    * ensure that only these types are used for inter-process communication

### Gateways

* [ ] Host2Host Communication based on <https://github.com/smoltcp-rs/smoltcp>
* [ ] mqtt (rumqtt)
* [ ] dds (rustdds or dustdds)
* [ ] zenoh
* [ ] someip (maybe sommr)
* [ ] dbus (zbus)

### Microservices (Quality of Life Improvements)

#### iceoryx Tooling

* [ ] Service Discovery
* [ ] Introspection Service
* [ ] Process Monitor (process can register and cleans up resources when process
      dies)
* [ ] Health Monitor
* [ ] Basic command line introspection tooling
* [ ] Tooling for advanced introspection, cool WebGUI
* [ ] Command line client as interface to microservices

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
* [ ] Request Response Messaging Pattern
* [ ] Blackboard Messaging Pattern
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
    * all error classes should implement `std::error::Error` for the non `no_std`
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
