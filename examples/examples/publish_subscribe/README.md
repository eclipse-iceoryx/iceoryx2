# Publish-Subscribe

## Running The Example

Here's an improved description for the publish-subscribe example of Iceoryx2:

"This example vividly illustrates a robust publisher-subscriber communication
pattern between two separate processes. The publisher diligently sends two
messages every second, each containing essential [`TransmissionData`]. On the
receiving end, the subscriber diligently checks for new data every second.

Whenever new data arrives, the subscriber swiftly acknowledges it by printing
the received information to the console.

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
