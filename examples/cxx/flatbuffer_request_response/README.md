# FlatBuffers Request-Response

This example demonstrates how to use dynamically sized data and Flatbuffers
for request-response zero-copy communication with iceoryx2.

FlatBuffers are fully integrated into iceoryx2. This means that users can work
with the FlatBuffers API directly through the iceoryx2 API without having to
create a custom allocator or track the payload offset as the FlatBuffer grows
backwards.

All surrounding memory management is handled by iceoryx2, allowing users to
focus entirely on generating dynamically sized data with the FlatBuffers API.

In this example, we send a request of type `UnboundedData`, and receive a `DataProps`
response type. The types are defined in the `unbounded_data.fbs` and `data_props.fbs`
as follows:

## Data Types

### UnboundedData

```fbs
namespace Example;

table Entry {
    data_1: int32;
    data_2: uint64;
}

table UnboundedData {
    title: string;
    entries: [Entry];
}

root_type UnboundedData;
```

### DataProps

```fbs
namespace Example;

table DataProps {
    received_entries_len: uint64;
}

root_type DataProps;
```

## Prerequisites

To use the FlatBuffers example, first install the FlatBuffers package.

```sh
# Arch Linux
pacman -S flatbuffers

# Debian/Ubuntu
apt install libflatbuffers-dev
```

## Usage

The generated Rust code is already included in this example. For completeness,
the command used to generate it is documented below:

```sh
flatc -o examples/cxx/flatbuffer_request_response/src --cpp \
    examples/cxx/flatbuffer_request_response/src/unbounded_data.fbs
flatc -o examples/cxx/flatbuffer_request_response/src --cpp \
    examples/cxx/flatbuffer_request_response/src/data_props.fbs
```

By default, FlatBuffers support is disabled in iceoryx2. It can be enabled by
setting `IOX2_FEATURE_FLATBUFFERS` in cmake with:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON -DIOX2_FEATURE_FLATBUFFERS=ON
cmake --build target/ff/cc/build
```

To observe the communication in action, open two terminals and run the following
commands.

### Terminal 1

```sh
export IOX2_FLATBUFFER_SCHEMA_PATH="$(pwd)/examples/cxx/flatbuffer_request_response/src"
./target/ff/cc/build/examples/cxx/flatbuffer_request_response/example_cxx_flatbuffer_request_response_server
```

### Terminal 2

```sh
export IOX2_FLATBUFFER_SCHEMA_PATH="$(pwd)/examples/cxx/flatbuffer_request_response/src"
./target/ff/cc/build/examples/cxx/flatbuffer_request_response/example_cxx_flatbuffer_request_response_client
```

Feel free to run multiple instances of client or server processes
simultaneously to explore how iceoryx2 handles request-response
communication efficiently.

> [!TIP]
> You may hit the maximum supported number of ports when too many server or
> client processes run. Take a look at the
> [iceoryx2 config](../../../config) to set the limits globally or at the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.
