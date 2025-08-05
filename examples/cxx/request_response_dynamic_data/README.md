# Request-Response With Dynamic Data (Slice Of Shared Memory Compatible Types)

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

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C++ Examples Readme](../README.md).

First you have to build the C++ examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON
cmake --build target/ff/cc/build
```

## How to Run

To observe the communication in action, open two terminals and execute the
following commands:

### Terminal 1

```sh
./target/ff/cc/build/examples/cxx/request_response_dynamic_data/example_cxx_request_response_dyn_server
```

### Terminal 2

```sh
./target/ff/cc/build/examples/cxx/request_response_dynamic_data/example_cxx_request_response_dyn_client
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
