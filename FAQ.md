# Frequently Asked Questions

* [Tips and Tricks](#tips-and-tricks)
    * [How To Define Custom Data Types](#how-to-define-custom-data-types)
    * [Send Dynamic Data](#send-dynamic-data)
    * [Inter-Thread Communication](#inter-thread-communication)
    * [Interop Between 32-bit and 64-bit Processes](#interop-between-32-bit-and-64-bit-processes)
    * [Docker](#docker)
    * [Async API](#async-api)
    * [Change Log Backend](#change-log-backend)
    * [Supported Log Levels](#supported-log-levels)
    * [Subfolders Under /dev/shm](#subfolders-under-dev-shm)
    * [Custom Payload Alignment](#custom-payload-alignment)
    * [Accessing Services From Multiple Users](#accessing-services-from-multiple-users)
* [Error Handling](#error-handling)
    * [Something Is Broken, How To Enable Debug Output](#something-is-broken-how-to-enable-debug-output)
    * [Encountered a SEGFAULT](#encountered-a-segfault)
    * [Stack Overflow](#stack-overflow)
    * [100% CPU Load When Using The WaitSet](#100-cpu-load-when-using-the-waitset)
    * [SIGBUS Error](#sigbus-error)
    * [`PublishSubscribeOpenError(UnableToOpenDynamicServiceInformation)`](#publishsubscribeopenerrorunabletoopendynamicserviceinformation)
    * [Remove Stale Resources](#remove-stale-resources)
    * [Running Out of Memory](#running-out-of-memory)
    * [Running Out of File Descriptors](#running-out-of-file-descriptors)
    * [Internal Failure](#internal-failure)
        * [Maximum File-Descriptor Limit Exceeded](#maximum-file-descriptor-limit-exceeded)
    * [Losing Data](#losing-data)
    * [Losing Dynamic Data](#losing-dynamic-data)
    * [`iceoryx2-ffi-c` does not contain this feature: libc_platform](#iceoryx2-ffi-c-does-not-contain-this-feature-libc_platform)
    * [Service In Corrupted state](#service-in-corrupted-state)

## Tips And Tricks

### How To Define Custom Data Types

#### Rust

1. Ensure to only use data types suitable for shared memory communication like
   pod-types (plain old data, e.g. `u64`, `f32`, ...) or explicitly
   shared-memory compatible containers like some of the constructs in the
   `iceoryx2-bb-containers`.
2. Add `#[repr(C)]` to your custom data type so that it has a uniform memory
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

#### C++

1. Ensure to only use data types suitable for shared memory communication like
   pod-types (plain old data, e.g. `uint64_t`, `int32_t`, ...) or explicitly
   shared-memory compatible containers like some of the constructs in
   `iceoryx2-bb-cxx`.
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

### Send Dynamic Data

Take a look at the
[publish-subscribe dynamic data size example](examples/rust/publish_subscribe_dynamic_data).

### Inter-Thread Communication

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

#### Example

Take a look at the local_pubsub file in the

* [C service types example](examples/c/service_types)
* [C++ service types example](examples/cxx/service_types)
* [Python service types example](examples/python/service_types)
* [Rust service types example](examples/rust/service_types)

### Interop Between 32-bit And 64-bit Processes

This is currently not possible since we cannot guarantee to have the same layout
of the data structures in the shared memory. On 32-bit architectures 64-bit POD
are aligned to a 4 byte boundary but to a 8 byte boundary on 64-bit
architectures. Some additional work is required to make 32-bit and 64-bit
applications interoperabel.

### Docker

By default, a Docker container has a shared memory size of 64 MB. This can be
exceeded quickly, since iceoryx2 always pre-allocates memory for the worst-case
scenario.

The shared memory size can be adjusted with the `--shm-size=` parameter, see:
[https://docs.docker.com/engine/containers/run/#runtime-constraints-on-resources](https://docs.docker.com/engine/containers/run/#runtime-constraints-on-resources)

### Async API

Currently, iceoryx2 does not provide an async API but it is
[on our roadmap](https://github.com/eclipse-iceoryx/iceoryx2/issues/47).
However, we offer an event-based API to implement push notifications. For more
details, see the [event example](examples/rust/event).

### Changing Log Backend

The `iceoryx2` crate automatically configures the console logger as the
default logger backend, however it is possible to change this using feature
flags on the `iceoryx2-bb-loggers` crate.

The following loggers are available:

1. **console** - outputs log messages to the console
1. **buffer** - outputs log messages to a buffer
1. **file** - outputs log messages to a file
1. **log** - utilizes the `log` crate
1. **tracing** - utilizes the `tracing` crate

The feature can be set in a project's `Cargo.toml`:

```toml
iceoryx2-bb-loggers = { version = "0.1.0", features = ["std", "file"]}
```

Or specified when building the crate:

```console
cargo build --features iceoryx2-bb-loggers/std -features iceoryx2-bb-loggers/file
```

Alternatively, a custom logger backend can be set at runtime at the very
beginning of your application:

```
use iceoryx2::prelude::*;

static LOGGER: MyLogger = MyLogger::new();

fn main() {
    set_logger(&LOGGER);
    // ...
}
```

### Supported Log Levels

iceoryx2 supports different log levels
`trace`, `debug`, `info`, `warning`, `error`, `fatal`

### Subfolders Under: dev shm

By default, the `shm_open` syscall does not support subfolders. However, when
creating files in an in-memory filesystem, we can achieve the same performance
on Linux and customize the shared memory file location.

See the
[service variant customization example](examples/rust/service_variant_customization)
for a detailed tutorial.

### Custom Payload Alignment

Some libraries (especially SIMD-related code) require stricter alignment than
the type you’re using for communication provides. In those cases, you can
increase the payload alignment with the builder parameter `payload_alignment()`.

```rust
let service = node
    .service_builder(&"My/Funk/ServiceName".try_into()?)
    .publish_subscribe::<TransmissionData>()
    .payload_alignment(Alignment::new(1024).unwrap())
    .open_or_create()?;
```

### Accessing Services From Multiple Users

Currently, iceoryx2 does not yet implement access rights management for
services and therefore only the user, under which the process runs, can access
the underlying resources. Processes started under a different user will receive
an insufficient permissions error when opening them.

With the `dev_permissions` feature flag, iceoryx2 will make all resources
always globally accessible. But this is a development feature and shall not be
used in production! It is a short-term mitigation until iceoryx2 implements
rights management for services.

* **cargo**

    ```sh
    cargo build --features dev_permissions
    ```

* **Cargo.toml**

    ```sh
    iceoryx2 = { version = "X.Y.Z", features = ['dev_permissions'] }
    ```

* **CMake**

    ```sh
    cmake -S . -B target/ff/cc/build -DIOX2_FEATURE_DEV_PERMISSIONS=On
    ```

## Error Handling

### Something Is Broken, How To Enable Debug Output

#### Setting the `LogLevel`

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

### Linking Error - undefined symbol `__internal_default_logger`

The logger front-end retrieves the selected default logger by calling
a function provided by the `iceoryx2-bb-loggers` crate. If this crate is not
linked against when building an application, a linking error of this form will
be encountered:

```console
error: undefined symbol: __internal_default_logger
```

If using the `iceoryx2` crate as a dependency, this is handled automatically,
however if using a lower-level crate (such as `iceoryx2-cal` or one from
`iceoryx2-bb`) the following is required:

1. Include `iceoryx2-bb-loggers` as a dependency with the corresponding feature
   for your platform:
    ```toml
    iceoryx2-bb-loggers = { version = "x.y.z", features = ["std", "console"] }
    ```
1. Ensure the crate is linked to even if not used:
    ```rust
    extern crate iceoryx2_loggers;
    ```

### Encountered a SEGFAULT

**What Kind Of Data Types Can Be Transmitted Via iceoryx2?**

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

### Stack Overflow

Most likely your payload type is too large and you need to construct the type
in-place of the shared memory.

Take a look at the
[complex data types example](examples/rust/complex_data_types).

In this example the `PlacementDefault` trait is introduced that allows in place
initialization and solves the stack overflow issue when the data type is larger
than the available stack size.

### 100% CPU Load When Using The WaitSet

The WaitSet wakes up whenever an attachment, such as a `Listener` or a `socket`,
has something to read. If you do not handle all notifications, for example, with
`Listener::try_wait_one()`, the WaitSet will wake up immediately again,
potentially causing an infinite loop and resulting in 100% CPU usage.

### SIGBUS Error

This error is usually caused by insufficient memory. When iceoryx2 allocates
shared memory, the operating system can overcommit by allocating more memory
than is physically available. As soon as the process actually accesses this
overcommitted memory, a `SIGBUS` signal is raised. This can occur with both
fixed-size and dynamic messages such as slices.

When using dynamic messages, the error may appear later, once your data size
grows and iceoryx2 needs to reallocate memory. For this reason, you should
avoid relying on dynamic memory in critical systems.

#### Check

You can check how much memory iceoryx2 is currently using by inspecting the
shared memory objects in /dev/shm:

```sh
du -sh /dev/shm
```

### `PublishSubscribeOpenError(UnableToOpenDynamicServiceInformation)`

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

You can manually [remove stale resources](#remove-stale-resources) to cleanup
your system.

### Remove Stale Resources

There are three different approaches to initiate stale resource cleanup:

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

### Running Out of Memory

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

### Running Out of File Descriptors

When using many services in a single process or across the system, a process can
hit the maximum number of open file descriptors. On Linux, you can increase this
limit temporarily with:

```sh
ulimit -n 8192
```

Or permanently by modifying `/etc/security/limits.conf`:

```text
*      soft      nofile      4096
*      hard      nofile      8192
```

### Running Out of Memory Mappings

When using many services in a single process or across the system, a process can
hit the maximum number of memory mappings. On Linux it is usually 65535 and you
can temporarily increase it as `root` with:

```sh
echo 1048576 > /proc/sys/vm/max_map_count
```

Or permanently by modifying `/etc/sysctl.conf`:

```text
vm.max_map_count=1048576
```

### Internal Failure

#### Maximum File-Descriptor Limit Exceeded

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

### Losing Data

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

### Losing Dynamic Data

If you're sending slices and have defined an allocation strategy, a sample may
be lost if the sender shuts down after reallocating its data segment. This is
because no receiver has yet mapped the reallocated data segment. Therefore, the
sender closes the data segment when going out of scope.

To circumvent this, you could use the size of the last sent sample as
`initial_max_slice_len` and use the `Static` allocation strategy.

<!-- markdownlint-disable MD044 'c' needs to be lower-case -->
### `iceoryx2-ffi-c` Does Not Contain This Feature: `libc_platform`
<!-- markdownlint-enable MD044 -->

In order to use the `iceoryx2` feature flags when building the `iceoryx2-ffi-c`
crate standalone, you needs to prefix the feature with `iceoryx2/`,
e.g. `--features iceoryx2/libc_platform.`.

### Service In Corrupted State

This error can have multiple causes.

1. Crashed processes cleaned up resources incompletely, see: [Remove Stale Resources](#remove-stale-resources)
2. The processes are compiled with two incompatible iceoryx2 versions.
3. Two service variants were used that are not compatible.
