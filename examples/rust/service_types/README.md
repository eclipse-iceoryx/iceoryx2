# Service-Types

Service types allow the specialization of the underlying mechanisms of
iceoryx2. It is a powerful tool to customize the behavior of all internal
aspects. Let's assume you use iceoryx2 in a unit test suite which runs
concurrently on your CI. In those cases, it would be ideal when iceoryx2 would
not create actual inter-process resources like shared memory, which would
interfere with another process, like another concurrently running test suite.

Or if you want to share GPU memory between processes and want to enable
iceoryx2 to handle the zero-copy communication based on them. In embedded
contexts, you might want to communicate across hypervisor partitions or
between an ARM A-core to an R-core. In all of those situations, you would need
special mechanisms to use the underlying memory or to send event trigger
mechanisms.

With service types, you have the ability to use iceoryx2 in a different
scenario without changing a single line of code, except the one line that
defines the service type. The service type is set when the `Node` is
created with

```rust
let node = NodeBuilder::new()
    .create::<ipc::Service>()?;
```

By default, all examples are setting it to `ipc::Service`. Let's assume you
would like to use the intra-process specialization. Then you can use
`local::Service`. In this case, all mechanisms are strictly contained in the
process itself, and all services cannot be used or discovered outside of the
process.

Thanks to Rust's `Send` and `Sync` traits, we can ensure via the compiler that
non-threadsafe constructs are not used by accident in a concurrent setup.
By default, all ports (e.g., `Publisher`, `Subscriber`, `Server`, `Client`,
...) are not threadsafe and do not even implement `Send`, the same goes for
the payload (e.g., `Sample`, `Request`, ...). But sometimes you need to use
them in a multithreaded context and are prepared to pay the additional costs
of a mutex, in those cases, you can use `ipc_threadsafe::Service` or
`local_threadsafe::Service`.

## Local PubSub

This example uses iceoryx2 for inter-thread communication. It spawns a
background thread and creates a node for every thread (`main` and the
`background_thread`) to enable easy inter-thread communication.
The advantage is that you no longer have to manually handle your MPMC queue and
share it between threads.

### How To Run

```sh
cargo run --example service_types_local_pubsub
```

All services are strictly confined to the process. Check out the directories
`/tmp/iceoryx2` or `/dev/shm`. As you can see, there are no resources created.
Also calling

```sh
iox2 service list
```

will show that there are no discoverable services running.

## IPC Publisher & IPC Threadsafe Subscriber

The IPC publisher example is the one you are already familiar with from the
introductory publish-subscribe example. The one thing you will observe is that,
even though the publisher publishes on the same service, the `local_pubsub`
process will not receive any messages since it is confined to the process.

The IPC threadsafe subscriber example uses the service type
`ipc_threadsafe::Service`, which makes every port threadsafe and therefore can
be shared between threads safely. To demonstrate this, we create another thread
and loop in the main- and the background-thread for messages.

### How To Run

#### Terminal 1

```sh
cargo run --example service_types_ipc_publisher
```

#### Terminal 2

```sh
cargo run --example service_types_ipc_threadsafe_subscriber
```

If you now check out the directories `/tmp/iceoryx2` or `/dev/shm`, you will
see the resources both inter-process communicating processes have created.
Also, a call to

```sh
iox2 service list
```

will discover the running service.

## Summary

* `ipc::Service` - inter-process communication
* `ipc_threadsafe::Service` - inter-process communication, all ports implement
  `Send`+`Sync`, all payloads `Send` at the cost of an additional mutex
* `local::Service` - inter-thread communication, strictly confined to the process
* `local_threadsafe::Service` - inter-thread communication, all ports implement
  `Send`+`Sync`, all payloads `Send` at the cost of an additional mutex

Defined when creating a `Node` and all constructed `Service`s created by that
`Node` will use the specified service type.

```rust
let node = NodeBuilder::new()
    .create::<ipc::Service>()?;
```
