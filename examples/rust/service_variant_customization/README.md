# Service Variant Customization

iceoryx2 allows customizing its internal communication mechanisms through
_service types_. This feature enables adapting iceoryx2 to different
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

In all these scenarios, service types allow you to plug in the appropriate
underlying communication mechanism.

## Customize Service Variant

The service type is specified when creating a `Node`:

```rust
let node = NodeBuilder::new()
    .create::<ipc::Service>()?;
```

## Example: Publisher

This example demonstrates inter-thread communication using `local::Service`. A
node is created per thread (`main` and a background thread), enabling
communication between them without manual MPMC queue handling.

### Run It

```sh
cargo run --example service_types_local_pubsub
```

Since all services are confined to the process:

* No shared memory or external resources are created (check `/tmp/iceoryx2` or
  `/dev/shm`).
* Running `iox2 service list` will show **no discoverable services**.

## Example: Subscriber

These examples use inter-process communication and show how service types affect
service visibility and thread safety.

* The **IPC Publisher** (`ipc::Service`) works like the default pub-sub example.
* The **IPC Threadsafe Subscriber** uses `ipc_threadsafe::Service`, making all
  ports thread-safe.

To demonstrate thread safety, this subscriber launches an additional thread that
also listens for messages.

### Run It

#### Terminal 1 (Publisher)

```sh
cargo run --example service_types_ipc_publisher
```

#### Terminal 2 (Threadsafe Subscriber)

```sh
cargo run --example service_types_ipc_threadsafe_subscriber
```

After starting both:

* You’ll see shared memory resources in `/tmp/iceoryx2` or `/dev/shm`.
* Running `iox2 service list` will list the discoverable services.

Note: The local pubsub process will **not receive** messages from the IPC
publisher, as it's confined to the process.
