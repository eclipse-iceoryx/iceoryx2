# Service-Types

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

```cxx
auto node = NodeBuilder()
    .create<ServiceType::Ipc>()
```

By default, all examples use `iox2.ServiceType.Ipc`. You can swap in a different
service type depending on your needs:

* `ServiceType::Ipc` – Default; for inter-process communication. All ports
  are thread-safe.
* `ServiceType::Local` – For intra-process communication; services are
  limited to the current process. All ports are thread-safe.

In contrast to Rust, C++ does not have terminology that corresponds
to `Send` and `Sync` traits, and cannot ensure that
non-thread-safe objects are not accidentally shared across threads. Therefore,
by default, all ports like `Publisher`, `Subscriber`, `Server`, and `Client`
are **thread-safe**.

## Example: Local PubSub

This example demonstrates inter-thread communication using
`ServiceType::Local`. A node is created per thread (`main` and a background
thread), enabling communication between them without manual MPMC queue handling.

### How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C++ Examples Readme](../README.md).

First you have to build the C++ examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON
cmake --build target/ff/cc/build
```

### How To Run

```sh
./target/ff/cc/build/examples/cxx/service_types/example_cxx_service_types_local_pubsub
```

Since all services are confined to the process:

* No shared memory or external resources are created (check `/tmp/iceoryx2` or
  `/dev/shm`).
* Running `iox2 service list` will show **no discoverable services**.

## IPC Publisher & IPC Threadsafe Subscriber

These examples use inter-process communication and show how service types affect
service visibility and thread safety.

* The **IPC Publisher** (`ServiceType::Ipc`) works like the default pub-sub example.
* The **IPC Threadsafe Subscriber** uses `ServiceType::Ipc`, and demonstrates the
  ports thread safety.

To demonstrate thread safety, this subscriber launches an additional thread that
also listens for messages.

### How To Run

#### Terminal 1

```sh
./target/ff/cc/build/examples/cxx/service_types/example_cxx_service_types_ipc_publisher
```

#### Terminal 2

```sh
./target/ff/cc/build/examples/cxx/service_types/example_cxx_service_types_ipc_threadsafe_subscriber
```

After starting both:

* You’ll see shared memory resources in `/tmp/iceoryx2` or `/dev/shm`.
* Running `iox2 service list` will list the discoverable services.

Note: The local pubsub process will **not receive** messages from the IPC
publisher, as it's confined to the process.

## Summary

| Service Type         | Scope         | Thread Safety     | Notes                                               |
| -------------------- | ------------- | ----------------- | --------------------------------------------------- |
| `ServiceType::Ipc`   | Inter-process | ✅ Thread-safe     | Adds mutex overhead for safe sharing across threads |
| `ServiceType::Local` | Intra-process | ✅ Thread-safe     | Safe for multi-threaded intra-process communication |

All ports (`Publisher`, `Subscriber`, etc.) and payloads (`Sample`, `Request`,
etc.) are affected by the service type defined when the `Node` is created.

```cxx
auto node = NodeBuilder()
    .create<ServiceType::Ipc>()
```
