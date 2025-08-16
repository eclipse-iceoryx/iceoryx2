# Request-Response Example in C

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation, ensuring that shared structs have the
>     same data layout
> * not use pointers to manage their internal structure

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

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C Examples Readme](../README.md).

First you have to build the C examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON -DBUILD_CXX=OFF
cmake --build target/ff/cc/build
```

## How to Run

To observe the communication in action, open two terminals and execute the
following commands:

### Terminal 1

```sh
./target/ff/cc/build/examples/c/request_response/example_c_request_response_client
```

### Terminal 2

```sh
./target/ff/cc/build/examples/c/request_response/example_c_request_response_server
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
