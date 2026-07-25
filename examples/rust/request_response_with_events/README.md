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
>
> **Only fixed-size integers (like `u8`), floating-point types (`f32` and**
> **`f64`), and the types in the `iceoryx2-bb-container` library are**
> **cross-language compatible!**

This example demonstrates how events can be used in combination with the
the request-response messaging pattern to notify only the `Client` that
actually receives the response.

## Client Side

The `Client` uses the following approach:

1. A `Notifier` to wake up the `Server` and a `Listener` to wait for its response,
   is created
2. A `Client` is created from a `Service` with a custom `UserHeader` to transmit
   the `ListenerId` it is waiting on
3. The `Client` loans memory and sets the request payload and user header
4. Sends the `RequestMut` to the `Server` and waits on the `Listener` for the
   response

## Server Side

The `Server` uses the following approach:

1. A `Notifier` to wake up the `Client` and a `Listener` to wait for its requests,
   is created
2. A `Server` is created from a `Service` with a custom `UserHeader` to identify
   the `ListenerId` it needs to notify
3. The `Server` waits on the `Listener` for new requests
4. When a request is received, it sends the response and notifies the `Client`
   with the `ListenerId` it got from the custom `UserHeader`

With this approach, event based communication can be used with request-response
in an efficient way.

## How to Run

To observe the communication in action, open three terminals and execute the
following commands:

### Terminal 1

```sh
cargo run --example request_response_with_events_server
```

### Terminal 2

```sh
cargo run --example request_response_with_events_client
```

### Terminal 3

```sh
cargo run --example request_response_with_events_client
```

The output on the `Client` terminals should always print `1` for the number of
received notifications. Feel free to replace the notification for a single
`listener` with a generic `notify` to observe that the `Clients` get more than
one notification per request.

> [!TIP]
> You may hit the maximum supported number of ports when too many client or
> server processes are running. Refer to the [iceoryx2 config](../../../config)
> to configure limits globally, or use the
> [Service builder API](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a specific service.
