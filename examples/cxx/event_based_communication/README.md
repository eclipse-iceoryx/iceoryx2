# Event-Based Communication

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation, ensuring that shared structs have the
>     same data layout
> * not use pointers to manage their internal structure
>
> Data types like `std::string` or `std::vector` will cause undefined behavior
> and may result in segmentation faults. We provide alternative data types
> that are compatible with shared memory. See the
> [complex data type example](../complex_data_types) for guidance on how to
> use them.

This example demonstrates iceoryx2's event multiplexing mechanism in a more
complex setup. The iceoryx2 `Publisher` and `Subscriber` are integrated into
custom `ExamplePublisher` and `ExampleSubscriber` classes, which also
incorporate an additional iceoryx2 `Notifier` and `Listener`. This setup
enables automatic event emission whenever an `ExamplePublisher` or
`ExampleSubscriber` is created or dropped. Additionally, events are emitted
whenever a new `Sample` is sent or received.

When a `class` inherits from `FileDescriptorBased`, it can be attached to a
`WaitSet`. Both `ExamplePublisher` and `ExampleSubscriber` implement this
interface by forwarding calls to their underlying `Listener`, which already
provides an implementation of `FileDescriptorBased`.

The `WaitSet` notifies the user of the origin of an event notification. The
user can then acquire the `EventId` from the `Listener`. Based on the value of
the `EventId`, the user can identify the specific event that occurred and take
appropriate action.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C++ Examples Readme](../README.md).

First you have to build the C++ examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON
cmake --build target/ff/cc/build
```

## How to Run

### Terminal 1

```sh
./target/ff/cc/build/examples/cxx/event_based_communication/example_cxx_event_based_communication_publisher
```

### Terminal 2

```sh
./target/ff/cc/build/examples/cxx/event_based_communication/example_cxx_event_based_communication_subscriber
```

Feel free to run multiple publishers or subscribers in parallel.
