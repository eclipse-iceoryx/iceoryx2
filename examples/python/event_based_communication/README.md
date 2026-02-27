# Event-Based Communication

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation, ensuring that shared structs have the
>     same data layout
>     * therefore, only `ctypes` and `ctypes.Structure` can be transferred
> * not use pointers to manage their internal structure
>
> Any other python data type, except `ctypes` or `ctypes.Structure`s, will
> likely cause undefined behavior and may result in segmentation faults.
>
> **Only fixed-size integers (like `ctypes.c_uint8_t`), `ctypes.c_float`,**
> **`ctypes.c_double`, and the types in the `iceoryx2-bb-container` library**
> **are cross-language compatible!**

This example is a minimal Python version of the Rust/C++
`event_based_communication` example.
It combines:

* `publish_subscribe` for transporting `TransmissionData`
* `event` for signaling state changes
* `waitset` for event-driven processing

The publisher emits one sample every second and notifies `SentSample` (event id
`4`). The subscriber reacts to this event, drains all available samples, and
sends back `ReceivedSample` (event id `5`) as an acknowledgement.

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

To observe this communication in action, open two terminals and execute the
following commands:

### Terminal 1

```sh
poetry --project iceoryx2-ffi/python run python examples/python/event_based_communication/subscriber.py
```

### Terminal 2

```sh
poetry --project iceoryx2-ffi/python run python examples/python/event_based_communication/publisher.py
```

Optional: for isolated runs, you can override the service name in both
processes:

```sh
IOX2_SERVICE_NAME="My/Funk/ServiceName-Test" poetry --project iceoryx2-ffi/python run python examples/python/event_based_communication/subscriber.py
IOX2_SERVICE_NAME="My/Funk/ServiceName-Test" poetry --project iceoryx2-ffi/python run python examples/python/event_based_communication/publisher.py
```

> [!TIP]
> You may hit the maximum supported number of ports when too many publisher or
> subscriber processes run. Take a look at the
> [iceoryx2 config](../../../config) to set the limits globally or at the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.
