# Discovery

## Running The Example

This example demonstrates how to list all active services within your iceoryx2
application. To get the most out of this example, you can combine it with other
examples, such as the [event example](../event/) or the
[publish-subscribe example](../publish_subscribe/), to see active services in
action.

First you have to build the C++ examples:

```sh
cmake -S . -B target/ffi/build -DBUILD_EXAMPLES=ON
cmake --build target/ffi/build
```

To begin, let's start some interesting services. Open two terminals and execute
the following commands:

**Terminal 1**

```sh
./target/ffi/build/examples/cxx/event/example_c_event_listener
```

**Terminal 2**

```sh
./target/ffi/build/examples/cxx/publish_subscribe/example_c_publish_subscribe_subscriber
```

Once these services are running, you can call the following command:

```sh
./target/ffi/build/examples/cxx/discovery/example_c_discovery
```

This will display the static service details of both the event and the
publish-subscribe service, giving you a comprehensive view of the active
services in your iceoryx2 application.
