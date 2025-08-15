# Cross-language Publish-Subscribe

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
> Any other python data type, except `ctypes` or `ctypes.Structure`s, like will
> cause undefined behavior and may result in segmentation faults. Take a look
> at the [publish subscribe example](../publish_subscribe) to see how `ctypes`
> can be transferred.

This example illustrates a robust cross-language publisher-subscriber
communication pattern. You can find compatible applications in the
cross-language examples for every language that iceoryx2 supports. The publisher
applications of the cross-language examples send a message every second, each
containing `TransmissionData` and the `CustomHeader`. On the receiving end, the
subscriber applications of the cross-language examples print the received
payload and the user header to the console whenever new data arrives.

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

To observe the dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

Run the C++ subscriber application:

```sh
poetry --project iceoryx2-ffi/python run python examples/python/publish_subscribe_cross_language/subscriber.py
```

### Terminal 2

Run the C++ publisher application:

```sh
poetry --project iceoryx2-ffi/python run python examples/python/publish_subscribe_cross_language/publisher.py
```

Feel free to also run the subscriber and publisher applications from other
cross-language examples simultaneously to explore how iceoryx2 handles
publisher-subscriber communication between applications written in different
languages efficiently.

> [!TIP]
> You may hit the maximum supported number of ports when too many publisher or
> subscriber processes are running. Check the [iceoryx2 config](../../../config)
> to set the limits globally or refer to the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.

## How to enable cross-language publish-subscribe communication

To communicate with each other, publisher and subscriber applications must share
the same service configuration, including the payload and the user header type
name.

To allow cross-language communication involving C++ applications, iceoryx2
provides the possibility to customize the payload and the user header type name
by defining the method `type_name()` in the sent data struct and user header,
e.g.

```python
class TransmissionData(ctypes.Structure):
    _fields_ = [
        ("x", ctypes.c_int32),
        ("y", ctypes.c_int32),
        ("funky", ctypes.c_double),
    ]

    @staticmethod
    def type_name() -> str:
        """Returns the system-wide unique type name."""
        return "TransmissionData"


class CustomHeader(ctypes.Structure):
    _fields_ = [
        ("version", ctypes.c_int32),
        ("timestamp", ctypes.c_uint64),
    ]

    @staticmethod
    def type_name() -> str:
        """Returns the system-wide unique type name."""
        return "CustomHeader"
```

When the type names are set to the same value, and the structure has the same
memory layout, the python applications and applications written in other
supported languages can communicate.

> [!NOTE]
> For the communication with Rust applications, you don't need to provide
> `type_name()` for `ctypes.c_(u)int{8|16|32|64}`, `float`, `double` and `bool`
> payloads.
> These types are automatically translated into the Rust equivalents.

You can also send dynamic data between Python, C++ and Rust applications (see
[Publish-Subscribe With Dynamic Data](../publish_subscribe_dynamic_data)). If
you send `iox2.Slice`s of `ctypes.c_(u)int{8|16|32|64}`, `float`, `double` or
`bool`, the payload type name is automatically translated to the Rust
equivalent. For other slice types, you have to set `IOX2_TYPE_NAME` for the
inner type to the Rust equivalent to enable the communication.
