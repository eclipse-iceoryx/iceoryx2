# Health Monitoring

This example demonstrates how to create a robust system using iceoryx2.
A central daemon pre-creates all communication resources to ensure that every
required resource, such as memory, is available as soon as the application
starts.
Additionally, the subscriber is immediately informed if one of the processes
it depends on has crashed. Even if the central daemon itself crashes,
communication can continue without any restrictions. Thanks to the
decentralized API of iceoryx2, the subscriber can take over the role of the
central daemon and continue monitoring all processes.

The communication must also be reliable, and we expect publishers to provide
updates at regular intervals. If a publisher misses a deadline, we want to be
informed immediately. This situation can occur if the system is under heavy
load or if a process has crashed.

This example is more advanced and consists of four components:

* `central_daemon` - Must run first. It creates all communication resources and
    monitors all nodes/processes.
* `publisher_1` - Sends data at a specific frequency on `service_1`.
* `publisher_2` - Sends data at a specific frequency on `service_2`.
* `subscriber` - Connects to `service_1` and `service_2` and expects new samples
    within a specific time. If no sample arrives, it proactively checks for dead
    nodes.

```ascii
+----------------+   creates   ...........................
| central_daemon | ----------> : communication resources :
+----------------+             ...........................
  |                               ^
  |                   opens       |
  |             +-----------------+--------------+
  |             |                 |              |
  |   +-------------+    +-------------+    +------------+
  |   | publisher_1 |    | publisher_2 |    | subscriber |
  |   +-------------+    +-------------+    +------------+
  |             ^                   ^                 ^
  |  monitores  |                   |                 |
  +-------------+-------------------+-----------------+
```

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation -> `#[repr(C)]`
> * not use pointers to manage their internal structure
>
> Data types like `String` or `Vec` will cause undefined behavior and may
> result in segmentation faults. We provide alternative data types that are
> compatible with shared memory. See the
> [complex data type example](../complex_data_types) for guidance on how to
> use them.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C++ Examples Readme](../README.md).

First you have to build the C++ examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON
cmake --build target/ff/cc/build
```

## How to Run

For this example, you need to open five separate terminals.

### Terminal 1: Central Daemon - Create All Communication Resources

Run the central daemon, which sets up all communication resources and monitors
processes.

```sh
./target/ff/cc/build/examples/cxx/health_monitoring/example_cxx_health_monitoring_central_daemon
```

### Terminal 2: Publisher 1

Run the first publisher, which sends data on `service_1`.

```sh
./target/ff/cc/build/examples/cxx/health_monitoring/example_cxx_health_monitoring_publisher_1
```

### Terminal 3: Publisher 2

Run the second publisher, which sends data on `service_2`.

```sh
./target/ff/cc/build/examples/cxx/health_monitoring/example_cxx_health_monitoring_publisher_2
```

### Terminal 4: Subscriber

Run the subscriber, which listens to both `service_1` and `service_2`.

```sh
./target/ff/cc/build/examples/cxx/health_monitoring/example_cxx_health_monitoring_subscriber
```

### Terminal 5: Simulate Process Crashes

Send a `SIGKILL` signal to `publisher_1` to simulate a fatal crash. This
ensures that the process is unable to clean up any resources.

```sh
killall -9 example_cxx_health_monitoring_publisher_1
```

After running this command:

1. The `central_daemon` will detect that the process has crashed and print:
   ```ascii
   detected dead node: Some(NodeName { value: "publisher 1" })
   ```
   The event service is configured to emit a `PubSub::ProcessDied` event when a
   process is identified as dead.

2. On the `subscriber` side, you will see the message:
   ```ascii
   ServiceName { value: "service_1" }: process died!
   ```

3. Since `publisher_1` is no longer sending messages, the subscriber will also
    regularly print another message indicating that `service_1` has violated
    the contract because no new samples are being received.

Feel free to run multiple instances of publisher or subscriber processes
simultaneously to explore how iceoryx2 handles publisher-subscriber
communication efficiently.

> [!TIP]
> You may hit the maximum supported number of ports when too many publisher or
> subscriber processes run. Take a look at the
> [iceoryx2 config](../../../config) to set the limits globally or at the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.
