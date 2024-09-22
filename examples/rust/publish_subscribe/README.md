# Publish-Subscribe

## Running The Example

This example illustrates a robust publisher-subscriber communication pattern
between two separate processes. The publisher sends two messages every second,
each containing [`TransmissionData`]. On the receiving end, the subscriber
checks for new data every second.

The subscriber is printing the sample on the console whenever new data arrives.

To observe this dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

```sh
cargo run --example publish_subscribe_subscriber
```

### Terminal 2

```sh
cargo run --example publish_subscribe_publisher
```

Feel free to run multiple instances of publisher or subscriber processes
simultaneously to explore how iceoryx2 handles publisher-subscriber
communication efficiently.

You may hit the maximum supported number of ports when too many publisher or
subscriber processes run. Take a look at the [iceoryx2 config](../../../config)
to set the limits globally or at the
[API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
to set them for a single service.
