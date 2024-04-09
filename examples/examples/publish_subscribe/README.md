# Publish-Subscribe

## Running The Example

This example vividly illustrates a robust publisher-subscriber communication
pattern between two separate processes. The publisher diligently sends two
messages every second, each containing essential [`TransmissionData`]. On the
receiving end, the subscriber checks for new data every second.

The subscriber is printing the sample on the console whenever new data arrives.

To observe this dynamic communication in action, open two separate terminals
and execute the following commands:

**Terminal 1**

```sh
cargo run --example publish_subscribe_subscriber
```

**Terminal 2**

```sh
cargo run --example publish_subscribe_publisher
```

Feel free to run multiple instances of publisher or subscriber processes
simultaneously to explore how Iceoryx2 handles publisher-subscriber communication
efficiently.
