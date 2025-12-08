# Service Variant Customization

iceoryx2 allows customizing its internal communication mechanisms through
_service variants_. This feature enables adapting iceoryx2 to different
environments and use cases without modifying your application logic -
just change the service type.

For instance:

* In **unit tests** running in parallel on a CI system, you may want to avoid
  creating real inter-process resources like shared memory that could interfere
  with other tests.
* If you're **sharing GPU memory** across processes, you may want iceoryx2 to
  handle zero-copy communication using custom memory mechanisms.
* In **embedded systems**, you might need communication across hypervisor
  partitions or between heterogeneous cores (e.g., ARM A-core to R-core).

In all these scenarios, service variants allow you to plug in the appropriate
underlying communication mechanism.

## Customize Service Variant

The service variant is specified when creating a `Node`:

```rust
let node = NodeBuilder::new()
    .create::<ipc::Service>()?; // we use the service variant 'ipc::Service'
```

A service variant can be customized by defining a struct and implementing
`iceoryx2::service::Service` and
`iceoryx2::service::internal::ServiceInternal<YourServiceVariantName>`.
`iceoryx2::service::Service` is a list of type definitions that define which
operating system mechanisms are used for the iceoryx2 concepts used to define a
service, a port’s data segment, or trigger mechanisms where one process can wake
up another.

In this example, we defined a `custom_service_variant.rs` file and customized
all mechanisms that would normally use POSIX shared memory to use a file-based
approach. This has the advantage that the `global.root-path` parameter is
applied to every resource that is represented in the file system, but also the
drawback that, when the location does not point to an in-memory file system
like `tmpfs`, performance will degrade significantly. If it is an in-memory
file system like `tmpfs`, performance should be identical.

## Example: Publisher & Subscriber

This is based on the publish-subscribe example, but uses the custom service
variant.

### Run: Publisher (Terminal 1)

```sh
cargo run --example service_variant_customization_publisher
```

### Run: Subscriber (Terminal 2)

```sh
cargo run --example service_variant_customization_subscriber
```

After starting, you will observe that:

* there are no more iceoryx2 shared memory objects under `/dev/shm`.
* all files, including the shared memory, are stored under `/tmp/iceoryx2`.

When you start the default publish-subscribe example, you will again see the
POSIX shared memory objects listed under `/dev/shm`.

### Run: Publisher (Terminal 1)

```sh
cargo run --example publish_subscribe_publisher
```

### Run: Subscriber (Terminal 2)

```sh
cargo run --example publish_subscribe_subscriber
```
