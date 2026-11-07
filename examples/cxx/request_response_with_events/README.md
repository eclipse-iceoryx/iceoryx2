# Request-Response with Events

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation, ensuring that shared structs have the
>     same data layout
> * not use pointers to manage their internal structure
> * must be trivially destructible, see `std::is_trivially_destructible`
>
> Data types like `std::string` or `std::vector` will cause undefined behavior
> and may result in segmentation faults. We provide alternative data types
> that are compatible with shared memory. See the
> [complex data type example](../complex_data_types) for guidance on how to
> use them.
>
> **Only fixed-size integers (like `uint8_t`), `float`, `double`, and the**
> **types in the `iceoryx2-bb-container` library are cross-language**
> **compatible!**

This example combines IPC request-response with events to wake only the client
whose response is ready. Run one server and two clients to see that each client
receives exactly one notification per response.

## Client Side

1. Create a notifier for the server's event service and a listener for the
   clients' event service.
2. Create a request-response client with a custom request header containing
   the client's listener ID.
3. Loan a request, write its payload and listener ID, send it, and notify the
   server.
4. Keep the pending response alive while waiting for an event, then receive
   the responses to this request.

## Server Side

1. Create a listener for incoming request notifications and a notifier for
   response notifications.
2. Wait for an event, then receive all available requests.
3. Send a response before inspecting the request's listener ID.
4. Use `for_each_listener` to find that ID, call the callback's `Monofier` to
   notify only that client, and stop iterating. The callback views must be used
   within the callback's lifetime.

The C++ and Python versions use the same service names and a fixed 16-byte
request header (`RequestResponseWithEventsHeader`, alignment 1). C++ copies the
bytes from `UniqueListenerId::bytes()`; Python encodes its integer ID in native
byte order. They can communicate with each other on the same host. The Rust
version uses a `u128` header and separate service names.

## How to Run

Build the examples by following the [C++ setup instructions](../README.md).

### Terminal 1

```sh
./target/ff/cc/build/examples/cxx/request_response_with_events/example_cxx_request_response_with_events_server
```

### Terminal 2

```sh
./target/ff/cc/build/examples/cxx/request_response_with_events/example_cxx_request_response_with_events_client
```

### Terminal 3

```sh
./target/ff/cc/build/examples/cxx/request_response_with_events/example_cxx_request_response_with_events_client
```

Each client prints `number of received notifications from server: 1` and a
response labeled with its request number. Replacing the server's selective
notification with `client_notifier.notify()` broadcasts to all clients,
including clients that have no response ready.

The end-to-end test runs two clients concurrently alongside an independent
idle listener that sends no requests. Every client must receive exactly one
notification and its response; any notification reaching the idle listener
fails the test, including a broadcast that otherwise appears to work.

> [!TIP]
> Service limits restrict the number of clients, servers, listeners, and
> notifiers. Use the [iceoryx2 config](../../../config) or the service builder
> to adjust them when running more instances.
