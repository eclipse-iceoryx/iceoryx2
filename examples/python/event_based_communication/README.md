# Minimal Event-Driven Communication

This is a minimal Python version of the Rust/C++ `event_based_communication`
example.

It combines:

- `publish_subscribe` for data transport (`TransmissionData`)
- `event` for signaling
- `waitset` for event-driven processing

The publisher sends one sample every second and emits event id `4`
(`SentSample`). The subscriber listens for that event, drains all available
samples, and emits event id `5` (`ReceivedSample`) as an acknowledgement.

## How to Build

Before running examples, set up the Python environment and build bindings:

```sh
poetry --project iceoryx2-ffi/python install
poetry --project iceoryx2-ffi/python build-into-venv
```

## How to Run

Terminal 1:

```sh
poetry --project iceoryx2-ffi/python run python examples/python/event_based_communication/subscriber.py
```

Terminal 2:

```sh
poetry --project iceoryx2-ffi/python run python examples/python/event_based_communication/publisher.py
```

Optional: override the service name for isolated runs:

```sh
IOX2_SERVICE_NAME="My/Funk/ServiceName-Test" python examples/python/event_based_communication/subscriber.py
IOX2_SERVICE_NAME="My/Funk/ServiceName-Test" python examples/python/event_based_communication/publisher.py
```
