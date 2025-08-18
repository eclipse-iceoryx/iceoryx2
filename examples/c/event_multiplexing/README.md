# Event

> [!CAUTION]
> The WaitSet wakes up as soon as there is data to read. If the data
> is not consumed in the callback, the WaitSet will immediately wake
> up the process again, potentially causing an infinite loop and leading
> to 100% CPU usage.

This example demonstrates iceoryx2's event multiplexing mechanism,
called the `WaitSet`. It allows waiting, with a single call, on
multiple `Listener` ports as well as external file descriptor-based
events such as `sockets`.

In this setup, the `wait` process monitors two services, which the
user can specify via the command line option `-s` and `-t`.
The `notifier` can define the service to which it will send event
notifications using the `-s` option and specify the event ID with
the `-e` option.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C Examples Readme](../README.md).

First you have to build the C examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON -DBUILD_CXX=OFF
cmake --build target/ff/cc/build
```

## How to Run

In the example below, we are waiting for events on the services `fuu` and
`bar`. Service `fuu` is notified with event ID `123`, and service `bar` is
notified with event ID `456`.

### Terminal 1

```sh
./target/ff/cc/build/examples/c/event_multiplexing/example_c_event_multiplexing_wait fuu bar
```

### Terminal 2

```sh
./target/ff/cc/build/examples/c/event_multiplexing/example_c_event_multiplexing_notifier 123 fuu
```

### Terminal 3

```sh
./target/ff/cc/build/examples/c/event_multiplexing/example_c_event_multiplexing_notifier 456 bar
```

Feel free to instantiate multiple notifiers for the same service with the same
or different event id's. Or to for different services.

## Technical Details

The `iox2_waitset_t` utilizes `epoll`, `select`, or other event-multiplexing
mechanisms. Before the `iox2_waitset_t` can monitor a specific event, it must
first be attached using `iox2_waitset_attach_notification()`, which returns a
RAII `iox2_waitset_guard_t`. This `Guard` automatically detaches the attachment
when it is cleaned up with `iox2_waitset_guard_drop()`.

The `iox2_waitset_wait_and_process()` call requires a callback that is invoked
for each triggered attachment and provides the `iox2_waitset_attachment_id_h`.
The user can either use `iox2_waitset_attachment_id_has_event_from()` to
identify the object associated with the `iox2_waitset_attachment_id_h`.
