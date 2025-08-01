# Event

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
notifications using the first command line argument and specify the event ID
with the second argument.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
the detailed instructions in the [Python Examples Readme](../README.md).

First you have to create a python environment, install maturin and compile
iceoryx2 and the language bindings:

```sh
# create python development environment
# needs to be called only once
python -m venv .env

# enter environment
source .env/bin/activate # or source .env/bin/activate.fish

# install maturin
pip install maturin

# compile language bindings
maturin develop --manifest-path iceoryx2-ffi/python/Cargo.toml
```

## How to Run

In the example below, we are waiting for events on the services `fuu` and
`bar`. Service `fuu` is notified with event ID `123`, and service `bar` is
notified with event ID `456`.

### Terminal 1

```sh
python examples/python/event_multiplexing/wait.py fuu bar
```

### Terminal 2

```sh
python examples/python/event_multiplexing/notifier.py fuu 123
```

### Terminal 3

```sh
python examples/python/event_multiplexing/notifier.py bar 456
```

Feel free to instantiate multiple notifiers for the same service with the same
or different event id's. Or to for different services.

## Technical Details

The `WaitSet` utilizes `epoll`, `select`, or other event-multiplexing
mechanisms. Before the `WaitSet` can monitor a specific event, it must first be
attached using `WaitSet::attach_notification()`, which returns a RAII `Guard`.
This `Guard` automatically detaches the attachment when it goes out of scope.

The `WaitSet::wait_and_process()` call requires a closure that is invoked for
each triggered attachment and provides the `AttachmentId`. The user can either
use `AttachmentId::has_event_from($ATTACHED_OBJECT$)` to identify the object
associated with the `AttachmentId`, or set up an associative array
to quickly access the corresponding object.
