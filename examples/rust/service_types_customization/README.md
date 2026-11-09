# Service Type Customization

iceoryx2 allows customizing its internal communication mechanisms through
_service variants_. This feature enables adapting iceoryx2 to different
environments and use cases without modifying your application logic -
just change the service type.

For instance:

* When running **unit tests** running in parallel on a CI system, you may want
  to avoid creating real inter-process resources like shared memory that could
  interfere with other tests.
* If you're **sharing GPU memory** across processes, you may want iceoryx2 to
  handle zero-copy communication using custom memory mechanisms.
* In **embedded systems**, you might need communication across hypervisor
  partitions or between heterogeneous cores (e.g., ARM A-core to R-core).

In all these scenarios, service variants allow you to plug in the appropriate
underlying communication mechanism.

## Customize Service Types

The service type variant is specified when creating a `Node`:

```rust
let node = NodeBuilder::new()
    .create::<ipc::Service>()?; // we use the service type 'ipc::Service'
```

A service type can be customized by defining a struct
`YourServiceTypeName` and implementing
`iceoryx2::service::Service` and
`iceoryx2::service::internal::ServiceInternal<YourServiceTypeName>`.
`iceoryx2::service::Service` is a list of type definitions that specify which
operating system mechanisms are used for the iceoryx2 concepts used to define a
service, a port’s data segment, or trigger mechanisms where one process can wake
up another.

In this example, we defined a custom service in `custom_service/src/lib.rs` and
customized all mechanisms that would normally use POSIX shared memory to use a
file-based approach. This has the advantage that the `global.root-path` parameter
is applied to every resource that is represented in the file system, but also the
drawback that, when the location does not point to an in-memory file system
like `tmpfs`, performance will degrade significantly. If it is an in-memory
file system like `tmpfs`, performance should be identical.

## Example: Publisher & Subscriber

This is based on the publish-subscribe example, but uses the custom service
type.

### Run: Publisher (Terminal 1)

```sh
cargo run --example service_types_customization_publisher
```

### Run: Subscriber (Terminal 2)

```sh
cargo run --example service_types_customization_subscriber
```

After starting, you will observe that:

* there are no more iceoryx2 shared memory objects under `/dev/shm`.
* all files, including the shared memory, are stored under `/tmp/iceoryx2`.

When you start the [publish-subscribe example](../publish_subscribe),
you will again see the POSIX shared memory objects listed under `/dev/shm`.

## Build The Language Bindings With The Custom Service Type

The language bindings support only two hard-coded service types, which is
`IpcService` and `LocalService`. I a future release there will be an option
to manually add more, but for now, in order to use a customized service type
from a language binding, the definition of these two types need to be
overwritten. For more details see the
[iceoryx2-ffi-service-types README.md](../../../iceoryx2-ffi/service-types/README.md).

In order to use the custom service types from this example with, e.g.,
the C++ bindings, the following steps need to be performed:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON -DRUST_CUSTOM_FFI_SERVICE_TYPES="$(pwd)/examples/customization/service-types/ffi-service-types"
cmake --build target/ff/cc/build
```

Now, the C++ publish-subscribe example can be used together with the publisher
and subscriber from this example:

### Run: Rust Publisher (Terminal 1)

```sh
cargo run --example service_types_customization_publisher
```

### Run: C++ Subscriber (Terminal 2)

```sh
target/ff/cc/build/examples/cxx/publish_subscribe/example_cxx_publish_subscribe_subscriber
```
