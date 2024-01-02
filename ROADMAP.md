# Roadmap

## Moonshots

* [ ] high performance d-bus alternative (thanks to true zero-copy)
* [ ] `#![no_std]` on `nightly` on all tier 1 platforms
* [ ] `#![no_std]` on `stable` on all tier 1 platforms
* [ ] completely dynamic setup with dynamic shared memory

## Shared Memory Container & Types

* [ ] Make `iceoryx2_bb_container` public with announcement
* [ ] Create and document dynamic size container concept for shared memory and apply it
        to all existing containers: `ByteString`, `Vec`, `Queue`
    * Open Question: How can these containers be cloned, copied?
* [ ] Introduce additional containers: `HashMap`, `Tree`, `Set`, `List`
* [ ] Introduce elementary types, look into: `simple-si-units` crate
    * Add types like: memory size, percentage, strict percentage (0..100), data throughput, resolution
        (further types found in informatics)
* [ ] Add `derive` proc macro to ensure that only shm compatible types can be
    transferred via zero-copy

## Language Bindings

* [ ] C
* [ ] C++
* [ ] Lua
* [ ] Python
* [ ] Zig

## Building Blocks

* [ ] WaitSet - event multiplexer based on reactor pattern
* [ ] Introduce trait and proc macro to generate types that can be sent via shared memory
  * ensure that only these types are used for inter-process communication

## Gateways

* [ ] Host2Host Communication based on <https://github.com/smoltcp-rs/smoltcp>
* [ ] mqtt (rumqtt)
* [ ] dds (rustdds or dustdds)
* [ ] zenoh
* [ ] someip (maybe sommr)
* [ ] dbus (zbus)

## Microservices (Quality of Life Improvements)

### Iceoryx Tooling

* [ ] Service Discovery
* [ ] Introspection Service
* [ ] Process Monitor (process can register and cleans up resources when process dies)
* [ ] Health Monitor
* [ ] Basic command line introspection tooling
* [ ] Tooling for advanced introspection, cool WebGUI
* [ ] Command line client as interface to microservices

### Tools and Gadgets

* [ ] System Monitor (show CPU load etc. like top)

## Communication

* [x] publish subscribe
* [x] events
* [ ] Single Publisher Subscribe with history
* [ ] Multi Publisher without history (except there is a brilliant idea on how to realize it with history)
* [ ] Request Response Messaging Pattern
* [ ] Blackboard Messaging Pattern
* [ ] Pipeline Messaging Pattern
* [ ] PubSub, ReqRes, Pipeline variant that works with copies (poor mans mixed criticality)
* [ ] Zero-copy GPU communication with Cuda, NvSci, Vulkan
* [ ] Zero-copy across hypervisor partitions
* [ ] Zero-copy via QEMU ivshmem: <https://www.qemu.org/docs/master/system/devices/ivshmem.html>
* [ ] dmabuf support, see: https://blaztinn.gitlab.io/post/dmabuf-texture-sharing/
* [ ] Support dynamic sized types in a memory efficient manner
  * Buddy allocator for sender data-segment
  * Introduce runtime fixed-size types

## Robustness

* [ ] Add ability to recover samples when subscriber died
  * add sample tracker into ZeroCopyConnection
  * add detection when subscriber returns non-received samples
* [ ] Large Communication Setup Support
  * handle the restriction of a max amount of posix shared memory objects of an OS
  * add `iceoryx2_cal` implementations that are using the `SharedMemoryGroup`

## Platform Support

* [ ] Android
* [x] Linux
* [x] Windows
* [x] Mac Os
* [ ] iOS
* [ ] WatchOS
* [x] FreeBSD
* [ ] FreeRTOS
* [ ] QNX

## Hardware Support

* [x] x86_64
* [x] aarch64
* [ ] armv7
* [ ] x32
* [ ] risc-v

## Framework Integration

* [ ] ROS2 rmw binding
* [ ] dora-rs integration

## Safety & Security

* [ ] Mixed Criticallity setup, e.g. applications do not interfer with each other
* [ ] Sample Tracking when application crashes
* [ ] Identity and Access Management, e.g. service that create additional services
* [ ] Use Kani, Loom and Miri for tests of lock-free constructs

## Development

* [ ] Tracing integration for advanced performance measurements, see google chrome
        chrome://tracing/ and flame graphs
        See lttng, add trace points to the source code on the important functions

## Quality Of Life Improvements

* [ ] Evaluate and refactor basic error handling approach based on enums
  * all error classes should implement `std::error::Error` for the non `no_std` build  (maybe use `thiserror`)
  * should be compatible with `anyhow` and `eyre`
  * explore error pyramid concept
* [ ] Rename `enable_safe_overflow` into `set_safe_overflow` in `ServiceBuilder` `publish_subscribe`
  * or maybe rename it into behavior: queue and ringbuffer, get inspired by crossbeam queues
* [ ] Provide `[T]` (slice) as special transmission type for pub/sub
  * `loan_slice`, `loan_uninit_slice` and `loan_uninit_slice_with_alignment`
* [ ] QoS feature for blocking publisher or pub/sub failures to perform custom error handling or expert behavior
  * explore implementation as trait
  * explore implementation as callback
* [ ] Explore if it is useful to have the same service name for different messaging patterns
  * separate them via internal suffix/prefix
  * simple use case: pub/sub + event to notify subscriber to notify sample send
  * would reduce error handling: connect to service with wrong messaging pattern
* [ ] Implement Resizable SharedMemoryConcept that is able to extend the shared memory by adding additional posix shared memory objects

## Integration Into Other Projects

* [ ] Maybe Hyprland
