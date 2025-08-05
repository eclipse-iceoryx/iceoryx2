# Discovery

This example demonstrates how to list all active services within your iceoryx2
application. To get the most out of this example, you can combine it with other
examples, such as the [event example](../event/) or the
[publish-subscribe example](../publish_subscribe/), to see active services in
action.

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

To begin, let's start some interesting services. Open two terminals and execute
the following commands:

### Terminal 1

```sh
poetry --project iceoryx2-ffi/python run python examples/python/event/listener.py
```

### Terminal 2

```sh
poetry --project iceoryx2-ffi/python run python examples/python/publish_subscribe/publisher.py
```

Once these services are running, you can call the following command:

```sh
poetry --project iceoryx2-ffi/python run python examples/python/discovery/discovery.py
```

This will display the static service details of both the event and the
publish-subscribe service, giving you a comprehensive view of the active
services in your iceoryx2 application.
