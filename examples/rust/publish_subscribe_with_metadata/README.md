# Publish-Subscribe With Metadata

## Running The Example

This example illustrates a publisher-subscriber communication pattern between two separate
processes with additional metadata, referred to as a `CustomHeader`. The publisher sends messages
every second, each containing an incrementing number and the `CustomHeader`, which includes an
additional version number and a timestamp. On the receiving end, the subscriber checks for new data
every second and prints out the received payload and metadata.

To observe this dynamic communication in action, open two separate terminals and execute the
following commands:

**Terminal 1**

```sh
cargo run --example publish_subscribe_metadata_subscriber
```

**Terminal 2**

```sh
cargo run --example publish_subscribe_metadata_publisher
```

Feel free to run multiple instances of the publisher or subscriber processes simultaneously to
explore how iceoryx2 handles publisher-subscriber communication efficiently.

You may hit the maximum supported number of ports when too many publisher or subscriber processes
are running. Check the [iceoryx2 config](../../../config) to set the limits globally or refer to
the [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html) to
set them for a single service.
