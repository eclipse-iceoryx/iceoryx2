# Event

## Running The Example

This example offers a practical demonstration of inter-process event signaling
in iceoryx2. It showcases how one process can signal an event to another
process, allowing for efficient communication.

In this scenario, the 'listener' process waits for incoming events. When an
event arrives, it promptly awakens and reports the [`EventId`] of the received
event. On the other end, the 'notifier' process periodically sends notifications
with an incrementing `EventId` every second.

First you have to build the C++ examples:

```sh
cmake -S . -B target/ffi/build -DBUILD_EXAMPLES=ON
cmake --build target/ffi/build
```

To see this in action, open two separate terminals and run the following
commands:

**Terminal 1**

```sh
./target/ffi/build/examples/cxx/event/example_cxx_event_listener
```

**Terminal 2**

```sh
./target/ffi/build/examples/cxx/event/example_cxx_event_notifier
```

Feel free to run multiple listeners or notifiers concurrently to observe how
iceoryx2 efficiently handles event signaling across processes.

You may hit the maximum supported number of ports when too many listener or
notifier processes run. Take a look at the [iceoryx2 config](../../../config) to set the
limits globally or at the
[API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
to set them for a single service.
