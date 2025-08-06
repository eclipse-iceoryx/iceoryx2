# Service Types in iceoryx2

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

## Choosing a Service Type

The service type is specified when creating a `Node`:

```rust
let node = NodeBuilder::new()
    .create::<ipc::Service>()?;
```

By default, all examples use `ipc::Service`. You can swap in a different
service type depending on your needs:

* `ipc::Service` – Default; for inter-process communication.
* `local::Service` – For intra-process communication; services are limited to
  the current process.
* `ipc_threadsafe::Service` – Like `ipc::Service`, but all ports implement
  `Send` + `Sync` using internal mutexes.
* `local_threadsafe::Service` – Like `local::Service`, but with thread-safe
  ports via mutexes.

Thanks to Rust’s `Send` and `Sync` traits, the compiler ensures that
non-thread-safe objects are not accidentally shared across threads. By default,
ports like `Publisher`, `Subscriber`, `Server`, and `Client`, as well as payload
types like `Sample` and `Request`, are **not thread-safe**. If you need thread
safety, use one of the `*_threadsafe::Service` types.

## Example: Local PubSub

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

## Example: IPC Publisher & Threadsafe Subscriber

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

## Summary of Service Types

| Service Type                | Scope         | Thread Safety     | Notes                                               |
| --------------------------- | ------------- | ----------------- | --------------------------------------------------- |
| `ipc::Service`              | Inter-process | ❌ Not thread-safe | Default for most examples                           |
| `ipc_threadsafe::Service`   | Inter-process | ✅ Thread-safe     | Adds mutex overhead for safe sharing across threads |
| `local::Service`            | Intra-process | ❌ Not thread-safe | Confined to the current process                     |
| `local_threadsafe::Service` | Intra-process | ✅ Thread-safe     | Safe for multi-threaded intra-process communication |

All ports (`Publisher`, `Subscriber`, etc.) and payloads (`Sample`, `Request`,
etc.) are affected by the service type defined when the `Node` is created.

### Example

```rust
let node = NodeBuilder::new()
    .create::<local_threadsafe::Service>()?;
```
