# Event Multiplexing

## Running The Example

This example demonstrates iceoryx2's event multiplexing mechanism,
called the `WaitSet`. It allows waiting, with a single call, on
multiple `Listener` ports as well as external file descriptor-based
events such as `sockets`.

In this setup, the `wait` process monitors an arbitrary number of
services, which the user can specify via the command line option `-s`.
The `notifier` can define the service to which it will send event
notifications using the `-s` option and specify the event ID with
the `-e` option.

In the example below, we are waiting for events on the services `fuu` and
`bar`. Service `fuu` is notified with event ID `123`, and service `bar` is
notified with event ID `456`.

### Terminal 1

```sh
cargo run --example event_multiplexing_wait -- -s "fuu" -s "bar"
```

### Terminal 2

```sh
cargo run --example event_multiplexing_notifier -- -s "fuu" -e 123
```

### Terminal 3

```sh
cargo run --example event_multiplexing_notifier -- -s "bar" -e 456
```

Feel free to instantiate multiple notifiers for the same service with the same
or different event id's. Or to for different services.

## Technical Details

The `WaitSet` utilizes `epoll`, `select`, or other event-multiplexing
mechanisms. Before the `WaitSet` can monitor a specific event, it must first be
attached using `WaitSet::attach()`, which returns a RAII `Guard`. This `Guard`
automatically detaches the attachment when it goes out of scope.

The `WaitSet::**_wait()` calls require a closure that is invoked for each
triggered attachment and provides the `AttachmentId`. The user can either use
`AttachmentId::originates_from($ATTACHED_OBJECT$)` to identify the object
associated with the `AttachmentId`, or set up a
`HashMap::<AttachmentId, Listener<ipc::Service>>` to quickly access the
corresponding object.
