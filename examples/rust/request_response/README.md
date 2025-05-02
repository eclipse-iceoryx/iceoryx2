# Request-Response

> [!CAUTION]
> Every payload you transmit with iceoryx2 must implement [`ZeroCopySend`] to
> be compatible with shared memory.
> Usually, you can use the derive-macro `#[derive(ZeroCopySend)]` for most
> types. If you implement it manually you must ensure that the payload type:
>
> * is self contained, no heap, no pointers to external sources
> * has a uniform memory representation -> `#[repr(C)]`
> * does not use pointers to manage their internal structure
> * and its members don't implement `Drop` explicitly
> * has a `'static` lifetime
>
> Data types like `String` or `Vec` will cause undefined behavior and may
> result in segmentation faults. We provide alternative data types that are
> compatible with shared memory. See the
> [complex data type example](../complex_data_types) for guidance on how to
> use them.

This example demonstrates the request-response messaging pattern between two
separate processes using iceoryx2. A key feature of request-response in
iceoryx2 is that the `Client` can receive a stream of responses instead of
being limited to just one.

## Client Side

The `Client` uses the following approach:

1. Sends first request by using the slower copy API and then enters a loop.
2. Inside the loop: Loans memory and acquires a `RequestMut`.
3. Writes the payload into the `RequestMut`.
4. Sends the `RequestMut` to the `Server` and receives a `PendingResponse`
   object. The `PendingResponse` can be used to:
   * Receive `Response`s for this specific `RequestMut`.
   * Signal the `Server` that the `Client` is no longer interested in data by
     going out of scope.
   * Check whether the corresponding `ActiveRequest` on the `Server` side is
     still connected.

## Server Side

The `Server` uses the following approach:

1. Receives the `RequestMut` sent by the `Client` and obtains an
   `ActiveRequest` object.
2. The `ActiveRequest` can be used to:
   * Read the payload, header, and user header.
   * Loan memory for a `ResponseMut`.
   * Signal the `Client` that it is no longer sending responses by going out
     of scope.
   * Check whether the corresponding `PendingResponse` on the `Client` side
     is still connected.
3. Sends one `Response` by using the slower copy API.
4. Loans memory via the `ActiveRequest` for a `ResponseMut` to send a response.

Sending multiple responses demonstrates the streaming API. The `ActiveRequest`
and the `PendingResponse` can call `is_connected()` to see if the corresponding
counterpart is still sending/receiving responses. As soon as the
`ActiveRequest` or `PendingResponse` went out-of-scope `is_connected()` will
return `false`.

In this example, both the client and server print the received and sent data
to the console.

## How to Run

To observe the communication in action, open two terminals and execute the
following commands:

### Terminal 1

```sh
cargo run --example request_response_server
```

### Terminal 2

```sh
cargo run --example request_response_client
```

Feel free to run multiple instances of the client or server processes
simultaneously to explore how iceoryx2 handles request-response communication
efficiently.

> [!TIP]
> You may hit the maximum supported number of ports when too many client or
> server processes are running. Refer to the [iceoryx2 config](../../../config)
> to configure limits globally, or use the
> [Service builder API](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a specific service.
