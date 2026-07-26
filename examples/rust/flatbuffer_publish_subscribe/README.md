# FlatBuffers Publish-Subscribe

This example demonstrates how to use dynamically sized data and FlatBuffers for
zero-copy communication with iceoryx2.

FlatBuffers are fully integrated into iceoryx2. This means that users can work
with the FlatBuffers API directly through the iceoryx2 API without having to
create a custom allocator or track the payload offset as the FlatBuffer grows
backwards.

All surrounding memory management is handled by iceoryx2, allowing users to
focus entirely on generating dynamically sized data with the FlatBuffers API.

In this example, we exchange the `UnboundedData` type, which is defined in the
`unbounded_data.fbs` file as follows:

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

## Prerequisites

To use the FlatBuffers example, first install the FlatBuffers package.

```sh
# Arch Linux
pacman -S flatbuffers

# Debian/Ubuntu
apt install flatbuffers
```

## Usage

The generated Rust code is already included in this example. For completeness,
the command used to generate it is documented below:

```sh
flatc -o examples/rust/flatbuffer_publish_subscribe --rust \
    examples/rust/flatbuffer_publish_subscribe/unbounded_data.fbs
```

To observe the communication in action, open two terminals and run the following
commands.

### Terminal 1

```sh
export IOX2_FLATBUFFER_SCHEMA_PATH="$(pwd)/examples/rust/flatbuffer_publish_subscribe"
cargo run --example flatbuffer_publisher
```

### Terminal 2

```sh
export IOX2_FLATBUFFER_SCHEMA_PATH="$(pwd)/examples/rust/flatbuffer_publish_subscribe"
cargo run --example flatbuffer_subscriber
```

Feel free to run multiple instances of publisher or subscriber processes
simultaneously to explore how iceoryx2 handles publisher-subscriber
communication efficiently.

> [!TIP]
> You may hit the maximum supported number of ports when too many publisher or
> subscriber processes run. Take a look at the
> [iceoryx2 config](../../../config) to set the limits globally or at the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.
