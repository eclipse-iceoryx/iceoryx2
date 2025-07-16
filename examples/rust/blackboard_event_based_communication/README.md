# Blackboard with Notification on Value Update

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

This example demonstrates how to combine the blackboard with the event messaging
pattern so that values can be read when updated instead of using a polling loop.
Besides the [blackboard](../blackboard) service, an additional [event](../event)
service is created. This is used to create a notifier that sends a notification
whenever a value is updated, using the entry id, and a listener that waits for
notifications with a certain entry id and reads the updated value.

## How to Run

To observe the blackboard messaging pattern with notifications on value update
in action, open two separate terminals and execute the following commands:

### Terminal 1

```sh
cargo run --example blackboard_event_based_creator
```

### Terminal 2

```sh
cargo run --example blackboard_event_based_opener
```
