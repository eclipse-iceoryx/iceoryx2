# Frequently Asked Questions

## Encountered a SEGFAULT. What Kind Of Data Types Can Be Transmitted Via iceoryx2

iceoryx2 stores all data in shared memory, which imposes certain restrictions.
Only data that is self-contained and does not use pointers to reference itself
is allowed. This is because shared memory is mapped at different offsets in each
process, rendering absolute pointers invalid. Additionally, if the data
structure uses the heap, it is stored locally within a process and cannot be
accessed by other processes. As a result, data types such as `String`, `Vec`, or
`HashMap` cannot be used as payload types.

Additionally, every data type must be annotated with `#[repr(C)]`. The Rust
compiler may reorder the members of a struct, which can lead to undefined
behavior if another process expects a different ordering.

To address this, iceoryx2 provides shared-memory-compatible data types. You can
refer to the [complex data types example](examples/rust/complex_data_types),
which demonstrates the use of `FixedSizeByteString` and `FixedSizeVec`.

## How To Define Custom Data Types (Rust)

1. Ensure to only use data types suitable for shared memory communication like
   pod-types (plain old data, e.g. `u64`, `f32`, ...) or explicitly
   shared-memory compatible containers like some of the constructs in the
   `iceoryx2-bb-containers`.
2. Add `#[repr(C`)]` to your custom data type so that it has a uniform memory
   representation.

   ```rust
    #[repr(C)]
    #[derive(ZeroCopySend)]
    struct MyDataType {
        //....
    }
   ```

3. **Do not use pointers, or data types that are not self-contained or use
   pointers for their internal management!**
4. The type and all of its members shall not implement `Drop`. The sender
   always owns the data and is responsible to cleanup everything. When it
   goes out of scope, the corresponding receivers must be able to still
   consume the sent data - therefore, iceoryx2 cannot call drop on maybe
   still in use data even when the process terminates.
   The internal iceoryx2 cleanup logic removes always all unused resources
   but when the last user is a receiver it is not able to mutate the
   memory and therefore cannot call drop on it.

## How To Define Custom Data Types (C++)

1. Ensure to only use data types suitable for shared memory communication like
   pod-types (plain old data, e.g. `uint64_t`, `int32_t`, ...) or explicitly
   shared-memory compatible containers like some of the constructs in the
   `iceoryx-hoofs`.
2. **Do not use pointers, or data types that are not self-contained or use
   pointers for their internal management!**
3. The type must be trivially destructible
   (see `std::is_trivially_destructible`). The sender always owns the data and
   is responsible to cleanup everything. When
   it goes out of scope, the corresponding receivers must be able to still
   consume the sent data - therefore, iceoryx2 cannot call the destructor on
   maybe still in use data even when the process terminates.
   The internal iceoryx2 cleanup logic removes always all unused resources
   but when the last user is a receiver it is not able to mutate the
   memory and therefore cannot call drop on it.

## How To Send Data Where The Size Is Unknown At Compilation-Time?

Take a look at the
[publish-subscribe dynamic data size example](examples/rust/publish_subscribe_dynamic_data).

## How To Use iceoryx2 For Inter-Thread Communication (As MPMC Queue Alternative)

iceoryx2 provides service variants optimized for different use cases. One such
variant is the local variant, which relies solely on mechanisms restricted to
the local process. For example, using the heap instead of shared memory.

By selecting the appropriate service variant, you can use iceoryx2 for
inter-thread communication as an alternative to MPMC queues. This approach
offers the added benefit of making it easy to extract threads into separate
processes later on, without needing to change anything except switching the
service variant from `Local` to `Ipc`.

* **Rust**

  ```rust
  let node = NodeBuilder::new()
       // or local_threadsafe::Service, or ipc::Service, or ipc_threadsafe::Service
       .create::<local::Service>()?;
  ```

* **Python**

  ```python
  // or iox2.ServiceType.Ipc
  node = iox2.NodeBuilder.new().create(iox2.ServiceType.Local)
  ```

* **C++**

  ```cxx
  auto node = NodeBuilder()
       // or Service::Ipc
      .create<ServiceType::Local>().expect("successful node creation");
  ```

* **C**

  ```c
  iox2_node_builder_h node_builder_handle = iox2_node_builder_new(NULL);
  iox2_node_h node_handle = NULL;
  // or iox2_service_type_e_IPC
  if (iox2_node_builder_create(node_builder_handle, NULL, iox2_service_type_e_LOCAL, &node_handle) != IOX2_OK) {
      printf("Could not create node!\n");
      goto end;
  }
  ```

### Example

Take a look at the local_pubsub file in the

* [C service variants example](examples/c/service_variants)
* [C++ service variants example](examples/cxx/service_variants)
* [Python service variants example](examples/python/service_variants)
* [Rust service variants example](examples/rust/service_variants)

## How To Make 32-bit and 64-bit iceoryx2 Applications Interoperatable

This is currently not possible since we cannot guarantee to have the same layout
of the data structures in the shared memory. On 32-bit architectures 64-bit POD
are aligned to a 4 byte boundary but to a 8 byte boundary on 64-bit
architectures. Some additional work is required to make 32-bit and 64-bit
applications interoperabel.

## My Transmission Type Is Too Large, Encounter Stack Overflow On Initialization

Take a look at the
[complex data types example](examples/rust/complex_data_types).

In this example the `PlacementDefault` trait is introduced that allows in place
initialization and solves the stack overflow issue when the data type is larger
than the available stack size.

## 100% CPU Load When Using The WaitSet

The WaitSet wakes up whenever an attachment, such as a `Listener` or a `socket`,
has something to read. If you do not handle all notifications, for example, with
`Listener::try_wait_one()`, the WaitSet will wake up immediately again,
potentially causing an infinite loop and resulting in 100% CPU usage.

## Does iceoryx2 Offer an Async API?

No, but it is
[on our roadmap](https://github.com/eclipse-iceoryx/iceoryx2/issues/47).
However, we offer an event-based API to implement push notifications. For more
details, see the [event example](examples/rust/event).

## Application does not remove services/ports on shutdown or several application restarts lead to port count exceeded

The structs of iceoryx2 need to be able to cleanup all resources when they go
out of scope. This is not the case when the application is:

* killed with the sigkill signal (`kill -9`)
* the `SIGTERM` signal is not explicitly handled

iceoryx2 already provides a mechanism that registers a signal handler that
handles termination requests gracefully, see
[publish subscribe example](examples/rust/publish_subscribe) and

```rust
while node.wait(CYCLE_TIME).is_ok() {
  // user code
}
```

But you can also use a crate like [ctrlc](https://docs.rs/ctrlc/latest/ctrlc/).

## How to use `log` or `tracing` as default log backend

* **log**, add the feature flag `logger_log` to the dependency in `Cargo.toml`
  ```toml
  iceoryx2 = { version = "0.1.0", features = ["logger_log"]}
  ```
* **tracing**, add the feature flag `logger_tracing` to the dependency in
  `Cargo.toml`
  ```toml
   iceoryx2 = { version = "0.1.0", features = ["logger_tracing"]}
  ```

## Supported log levels

iceoryx2 supports different log levels
`trace, debug, info, warning, error, fatal`

## How to set the log level

iceoryx2 provides different APIs which can be used to set the log level
directly in the code or read the configured log level by environment variable

`export IOX2_LOG_LEVEL=Trace`

then in the `main.rs` call one of the functions to set the log level

```rust
use iceoryx2::prelude::*

// ...

// Reads LogLevel from env and defaults to LogLevel INFO
// if the environment variable is not set or has an unsupported value
set_log_level_from_env_or_default();

// Reads LogLevel from env and sets it with a user-given value
// if the environment variable is not set or has an unsupported value
set_log_level_from_env_or(LogLevel::DEBUG);

// sets LogLevel programmatically with a supported user-given value
// and does not try to read the environment variable
set_log_level(LogLevel::Trace);
```

**Note**: While working on iceoryx2, it gets its default logging level from
`.cargo/config.toml`, but this can be over-ridden by using the APIs that reads
environment variable `IOX2_LOG_LEVEL` or set the log level directly in the code.

## A crash leads to the failure `PublishSubscribeOpenError(UnableToOpenDynamicServiceInformation)`

When an application crashes, some resources may remain in the system and need to
be cleaned up. This issue is detected whenever a new iceoryx2 instance is
created, removed, or when someone opens the service that the crashed process had
previously opened. On the command line, you may see a message like this:

```ascii
6 [W] "Node::<iceoryx2::service::ipc::Service>::cleanup_dead_nodes()"
      | Dead node (NodeId(UniqueSystemId { value: 1667667095615766886193595845
      | , pid: 34245, creation_time: Time { clock_type: Realtime, seconds: 172
      | 8553806, nanoseconds: 90404414 } })) detected
```

However, for successful cleanup, the process attempting the cleanup must have
sufficient permissions to remove the stale resources of the dead process. If the
cleanup fails due to insufficient permissions, the process that attempted the
cleanup will continue without removing the resources.

Generally, it is not necessary to manually clean up these resources, as other
processes should detect and handle the cleanup when creating or removing nodes,
or when services are opened or closed.

Nevertheless, there are three different approaches to initiate stale resource
cleanup:

1. **Using the iceoryx2 API**:

   ```rust
   Node::<ipc::Service>::list(Config::global_config(), |node_state| {
     if let NodeState::<ipc::Service>::Dead(view) = node_state {
       println!("Cleanup resources of dead node {:?}", view);
       if let Err(e) = view.remove_stale_resources() {
         println!("Failed to clean up resources due to {:?}", e);
       }
     }
     CallbackProgression::Continue
   })?;
   ```

2. **Using the command line tool**: `iox2 node -h` (NOT YET IMPLEMENTED)

3. **Manual cleanup**: Stop all running services and remove all shared memory
   files with the `iox2` prefix from:
   * POSIX: `/dev/shm/`, `/tmp/iceoryx2`
   * Windows: `c:\Temp\iceoryx2`

## Run Out-Of-Memory When Creating Publisher With A Large Service Payload

Since iceoryx2 is designed to operate in safety-critical systems, it must
ensure that a publisher (or sender) never runs out of memory. To achieve this,
iceoryx2 preallocates a data segment for each new publisher. This segment is
sized to handle the worst-case service requirements, ensuring that every
subscriber can receive and store as many samples as permitted, even under
maximum load.

To minimize the required worst-case memory, you can adjust specific service
settings. For example:

```rust
let service = node
    .service_builder(&"some service name".try_into()?)
    .publish_subscribe::<[u8]>()
    // Limit the maximum number of subscribers
    .max_subscribers(1)
    // Limit the number of samples a subscriber can hold simultaneously
    .subscriber_max_borrowed_samples(1)
    // Reduce the subscriber's overall sample buffer size
    .subscriber_max_buffer_size(1)
    // Limit the size of the publisher's history buffer
    .history_size(1)
    // ...
```

On the publisher side, you can also configure resource usage:

```rust
let publisher = service
    .publisher_builder()
    // Limit the number of samples a publisher can loan at once
    .max_loaned_samples(1)
    // ...
```

All these parameters can also be set globally by using the
[iceoryx2 config file](config).

## `Error: Resource Creation Failed`

### You May Exceeded The Maximum File-Descriptor Limit

When you increase the number of services or ports beyond a certain limit for
one process, the process may exceed the per-user file descriptor limit. This
limit can be increased by adjusting the `nofile` setting in the
`/etc/security/limits.conf` file:

```ascii
*     soft    nofile      4096
*     hard    nofile      8192
```

* `*` – Applies to all users
* `soft` | `hard` – The soft and hard limits
* The soft limit is set to 4096, while the hard limit is set to 8192

After making these changes, you can use the following command to increase the
soft file descriptor limit up to the hard limit:

```bash
ulimit -n <new_limit>
```

## Losing data

When a `Subscriber` or a `Server` does not seem to receive data, it may be
because the port was never able to connect to its counterpart - typically
because it went out of scope too quickly. For instance, this can happen when a
`Publisher` is created, sends some data, and is immediately destroyed
afterward.

Due to the decentralized nature of iceoryx2, and the fact that it does not use
any background threads, the user must handle these edge cases explicitly.

To address this, every port provides an `update_connections()` function. After
creating all ports, make sure to call `update_connections()` on each of them
before starting communication.

Another solution is to ensure that the sending ports remain alive at least
until the first piece of data has been successfully received.

## Losing a sample when sending dynamic data

If you're sending slices and have defined an allocation strategy, a sample may
be lost if the sender shuts down after reallocating its data segment. This is
because no receiver has yet mapped the reallocated data segment. Therefore, the
sender closes the data segment when going out of scope.

To circumvent this, you could use the size of the last sent sample as
`initial_max_slice_len` and use the `Static` allocation strategy.

<!-- markdownlint-disable MD044 'c' needs to be lower-case -->
## The package 'iceoryx2-ffi-c' does not contain this feature: libc_platform
<!-- markdownlint-enable MD044 -->

In order to use the `iceoryx2` feature flags when building the `iceoryx2-ffi-c`
crate standalone, you needs to prefix the feature with `iceoryx2/`,
e.g. `--features iceoryx2/libc_platform.`.
