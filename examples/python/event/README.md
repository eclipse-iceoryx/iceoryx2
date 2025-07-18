# Event

This example offers a practical demonstration of inter-process event signaling
in iceoryx2. It showcases how one process can signal an event to another
process, allowing for efficient communication.

In this scenario, the 'listener' process waits for incoming events. When an
event arrives, it promptly awakens and reports the [`EventId`] of the received
event. On the other end, the 'notifier' process periodically sends notifications
with an incrementing `EventId` every second.

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

To see this in action, open two separate terminals and run the following
commands:

### Terminal 1

```sh
python examples/python/event/listener.py
```

### Terminal 2

```sh
python examples/python/event/notifier.py
```

Feel free to run multiple listeners or notifiers concurrently to observe how
iceoryx2 efficiently handles event signaling across processes.

> [!TIP]
> You may hit the maximum supported number of ports when too many listener or
> notifier processes run. Take a look at the [iceoryx2 config](../../../config)
> to set the limits globally or at the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.
