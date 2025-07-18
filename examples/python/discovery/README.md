# Discovery

This example demonstrates how to list all active services within your iceoryx2
application. To get the most out of this example, you can combine it with other
examples, such as the [event example](../event/) or the
[publish-subscribe example](../publish_subscribe/), to see active services in
action.

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

To begin, let's start some interesting services. Open two terminals and execute
the following commands:

### Terminal 1

```sh
python examples/python/event/listener.py
```

### Terminal 2

```sh
python examples/python/publish_subscribe/publisher.py
```

Once these services are running, you can call the following command:

```sh
python examples/python/discovery/discovery.py
```

This will display the static service details of both the event and the
publish-subscribe service, giving you a comprehensive view of the active
services in your iceoryx2 application.
