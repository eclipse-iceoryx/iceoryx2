# Event

This example offers a practical demonstration of inter-process event signaling
in iceoryx2. It showcases how one process can signal an event to another
process, allowing for efficient communication.

In this scenario, the 'listener' process waits for incoming events. When an
event arrives, it promptly awakens and reports the [`EventId`] of the received
event. On the other end, the 'notifier' process periodically sends notifications
with an incrementing `EventId` every second.

## How to Build

Before proceeding, a virtual environment with all dependencies needs to be
created. You can find the detailed instructions in the
[Python Examples Readme](../README.md).

```sh
poetry --project iceoryx2-ffi/python install
```

Then, the iceoryx2 python bindings can be built and installed into the virtual
environment:

```sh
poetry --project iceoryx2-ffi/python run maturin develop --manifest-path iceoryx2-ffi/python/Cargo.toml --target-dir target/ff/python
```

## How to Run

To see this in action, open two separate terminals and run the following
commands:

### Terminal 1

```sh
poetry --project iceoryx2-ffi/python run python examples/python/event/listener.py
```

### Terminal 2

```sh
poetry --project iceoryx2-ffi/python run python examples/python/event/notifier.py
```

Feel free to run multiple listeners or notifiers concurrently to observe how
iceoryx2 efficiently handles event signaling across processes.

> [!TIP]
> You may hit the maximum supported number of ports when too many listener or
> notifier processes run. Take a look at the [iceoryx2 config](../../../config)
> to set the limits globally or at the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.

## Notify a Selected Listener

Run the self-contained LOCAL example to select a listener by name, notify it
inside a callback, and save its key for a later notification:

```sh
poetry --project iceoryx2-ffi/python run python examples/python/event/selective_notification.py
```

`for_each_listener` passes a `Monofier` and a `ListenerDetails` snapshot to each
callback. Return `CallbackProgression.Continue` or `CallbackProgression.Stop`.
Any other return value raises `TypeError`. Exceptions propagate unchanged after
cleanup; notifications already sent are not rolled back.

A `Monofier` is valid only on its creating thread, until that callback returns.
It never becomes valid again during another callback. Copy its `listener_key()`
to retain a target; keys and details can outlive the callback. A saved key does
not keep a listener alive, and notification may fail if that listener disappears.

During traversal, calls to the same notifier's methods or properties, including
`delete()`, raise `RuntimeError` on all threads. Use the callback's `Monofier`
for notification. Different notifiers remain usable. Outside traversal, ordinary
concurrent calls wait for each other; same-thread re-entry from an ordinary
operation (for example, a logging handler) raises `RuntimeError`.
