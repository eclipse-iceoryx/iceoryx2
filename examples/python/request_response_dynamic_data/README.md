# Request-Response With Dynamic Data (Slice Of Shared Memory Compatible Types)

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation, ensuring that shared structs have the
>     same data layout
>     * therefore, only `ctypes` and `ctypes.Structure` can be transferred
> * not use pointers to manage their internal structure
>
> Any other python data type, except `ctypes` or `ctypes.Structure`s, like will
> cause undefined behavior and may result in segmentation faults. Take a look
> at the [publish subscribe example](../publish_subscribe) to see how `ctypes`
> can be transferred.

This example demonstrates the dynamic memory request-response messaging pattern
between two separate processes using iceoryx2. A service with the request and
response payload type of an `uint8_t` slice is created, and every client and server
can define a slice length hint they support for communication with
`initial_max_slice_len`. The client sends a request with
increasing size every second containing a piece of dynamic data. On the
receiving end, the server checks for new data and sends a response with
increasing memory size.

The `initial_max_slice_len` hint and the `AllocationStrategy` set by the
client and server will define how memory is reallocated when

* [`Client::loan_slice()`], [`Client::loan_slice_uninit()`] on the client
  side or
* [`ActiveRequest::loan_slice()`], [`ActiveRequest::loan_slice_uninit()`] on
  the server side

request more memory than it is available.

## Client Side

The `Client` uses the following approach:

1. Sends first request and then enters a loop.
2. Inside the loop: Loans an increasing amount of memory and acquires a
  `RequestMut`.
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
3. Loans an increasing amount of memory via the `ActiveRequest` for a
  `ResponseMut` to send a response.

In this example, both the client and server print the amount of received bytes
to the console.

## How to Build

Before proceeding, a virtual environment with all dependencies needs to be
created. You can find the detailed instructions in the
[Python Examples Readme](../README.md).

```sh
poetry --project iceoryx2-ffi/python install
```

Then, the iceoryx2 python bindings can be built and installed into the virtual
environment:

```sh
poetry --project iceoryx2-ffi/python run maturin develop --manifest-path iceoryx2-ffi/python/Cargo.toml --target-dir target/ff/python
```

## How to Run

To observe the communication in action, open two terminals and execute the
following commands:

### Terminal 1

```sh
poetry --project iceoryx2-ffi/python run python examples/python/request_response_dynamic_data/server.py
```

### Terminal 2

```sh
poetry --project iceoryx2-ffi/python run python examples/python/request_response_dynamic_data/client.py
```

Feel free to run multiple instances of the client or server processes
simultaneously to explore how iceoryx2 handles request-response communication
efficiently.

> [!TIP]
> You may hit the maximum supported number of ports when too many client or
> server processes run. Take a look at the
> [iceoryx2 config](../../../config) to set the limits globally or at the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.
