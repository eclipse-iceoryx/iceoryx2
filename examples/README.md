# Examples

> [!TIP]
> In the examples root folder of every language you can find detailed
> installation instructions.

## Foundations of Communication in iceoryx2 Applications

> [!IMPORTANT]
> The examples are not yet functional in all languages. Check the list below to see
> what is already working!

In the world of iceoryx2, communication is at the core of everything. To
facilitate seamless communication, we always begin by opening or creating a
service, the fundamental building block of an iceoryx2 application. Services
serve as the conduit through which different parts of your application interact.

The type of service you create is defined by a messaging pattern, which dictates
how data flows between participants. iceoryx2 supports various messaging
patterns, including:

* **Publish-Subscribe:** In this pattern, a publisher sends a continuous stream
  of data to one or more subscribers, enabling real-time data dissemination.

* **Event:** This pattern allows notifiers to trigger events on a listener,
  which waits until it receives a notification. It is the basic pattern for
  implementing push-notifications.

* **Request-Response:** This pattern enables clients to send requests
  to a server, which responds with the requested data or action, making it
  suitable for interactive, transactional communication.

* **Blackboard (in progress):** This pattern realizes a key-value store in
  shared memory which can be modified by one writer and several readers.

* **Pipeline:** (planned) Borrowed from the Unix command line, this pattern
  involves a data source that produces data and transfers ownership to a sink,
  where it can be modified or processed in a pipeline-like fashion.

The service acts as a factory, creating service participants, often called
"ports." These ports establish communication links between different components
of your application. Each service can be customized with specific settings to
meet your application's requirements.

Within the service builder, you can configure various service-specific quality
of service parameters, ensuring that your communication behaves precisely as you
intend. The service port factory allows you to fine-tune the settings and
behavior of individual ports, giving you precise control over how they interact
and exchange data.

## Payload Type Restrictions

> [!CAUTION]
> iceoryx2 stores payload data in shared memory, which imposes the restriction that
> the payload type must be self-contained and cannot use heap memory. Additionally,
> internal pointers are not allowed because the shared memory is mapped at different
> offsets in each process, making absolute pointers invalid and potentially leading
> to segmentation faults.
>
> Furthermore, every data type must be annotated with `#[repr(C)]`. The Rust
> compiler may reorder the members of a struct, which can lead to undefined
> behavior if another process expects a different ordering.

To address these limitations, we provide data types that are compatible with shared
memory. For Rust, we offer:

* `FixedSizeByteString`
* `FixedSizeVec`

For C++, we provide:

* `iox::vector`
* `iox::string`
* `iox::list`

These types are demonstrated in the complex data types example.

## Overview

| Name                                 | Language                                                                                                                                                                                    | Description                                                                                                                                                                                                     |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| blackboard                           | [Rust](rust/blackboard)                                                                                                                                                                     | Unidirectional communication where one writer updates data in the blackboard which several readers can read.                                                                                                    |
| blackboard_event_based_communication | [Rust](rust/blackboard_event_based_communication)                                                                                                                                           | Blackboard communication where a writer sends notifications whenever a value is updated.                                                                                                                        |
| complex data types                   | [C++](cxx/complex_data_types) [Rust](rust/complex_data_types)                                                                                                                               | Send zero-copy compatible versions of `Vec` and `String`. Introduces `PlacementDefault` trait for large data types to perform an in place initialization where otherwise a stack overflow would be encountered. |
| discovery                            | [C](c/discovery) [C++](cxx/discovery) [Python](python/discovery) [Rust](rust/discovery)                                                                                                     | List all available services in a system.                                                                                                                                                                        |
| docker                               | [all](rust/docker)                                                                                                                                                                          | Communicate between different docker containers and the host.                                                                                                                                                   |
| domains                              | [C](c/domains) [C++](cxx/domains) [Python](python/domains) [Rust](rust/domains)                                                                                                             | Establish separate domains that operate independently from one another.                                                                                                                                         |
| event                                | [C](c/event) [C++](cxx/event) [Python](cxx/event) [Rust](rust/event)                                                                                                                        | Push notifications - send event signals to wakeup processes that are waiting for them.                                                                                                                          |
| event based communication            | [C++](cxx/event_based_communication) [Rust](rust/event_based_communication)                                                                                                                 | Define multiple events like publisher/subscriber created or removed, send sample, received sample, deliver history etc. and react on them for a fully event driven communication setup.                         |
| event multiplexing                   | [C](c/event_multiplexing) [C++](cxx/event_multiplexing) [Python](python/event_multiplexing) [Rust](rust/event_multiplexing)                                                                 | Wait on multiple listeners or sockets with a single call. The WaitSet demultiplexes incoming events and notifies the user.                                                                                      |
| health monitoring                    | [C++](cxx/health_monitoring) [Rust](rust/health_monitoring)                                                                                                                                 | A central daemon creates the communication resources and monitors all nodes. When the central daemon crashes other nodes can take over and use the decentral API to monitor the nodes.                          |
| publish subscribe                    | [C](c/publish_subscribe) [C++](cxx/publish_subscribe) [Python](python/publish_subscribe) [Rust](rust/publish_subscribe)                                                                     | Communication between multiple processes with a [publish subscribe messaging pattern](https://en.wikipedia.org/wiki/Publish–subscribe_pattern).                                                                 |
| publish subscribe cross language     | [C](c/publish_subscribe_cross_language) [C++](cxx/publish_subscribe_cross_language) [Python](python/publish_subscribe_cross_language) [Rust](rust/publish_subscribe_cross_language)         | Cross-language communication between multiple Rust, C++ and C processes with a [publish subscribe messaging pattern](https://en.wikipedia.org/wiki/Publish–subscribe_pattern).                                  |
| publish subscribe dynamic data       | [C++](cxx/publish_subscribe_dynamic_data) [Python](python/publish_subscribe_dynamic_data) [Rust](rust/publish_subscribe_dynamic_data)                                                       | Communication between multiple processes with a [publish subscribe messaging pattern](https://en.wikipedia.org/wiki/Publish–subscribe_pattern) and payload data that has a dynamic size.                        |
| publish subscribe with user header   | [C](c/publish_subscribe_with_user_header) [C++](cxx/publish_subscribe_with_user_header) [Python](python/publish_subscribe_with_user_header) [Rust](rust/publish_subscribe_with_user_header) | Add a user header to the payload (samples) to transfer additional information.                                                                                                                                  |
| request response                     | [C](c/request_response) [C++](cxx/request_response) [Python](python/request_response) [Rust](rust/request_response)                                                                         | Sending requests from one or many clients to one or many servers and receive a stream of responses.                                                                                                             |
| request response dynamic data        | [C++](cxx/request_response_dynamic_data) [Python](python/request_response_dynamic_data) [Rust](rust/request_response_dynamic_data)                                                          | Sending requests with increasing memory size from one or many clients to one or many servers and receive responses with increasing memory size.                                                                 |
| service attributes                   | [C](c/service_attributes) [C++](cxx/service_attributes) [Python](python/service_attributes) [Rust](rust/service_attributes)                                                                 | Creates a service with custom attributes that are available to every endpoint. If the attributes are not compatible the service will not open.                                                                  |
| service variants                     | [C](c/service_variants) [C++](cxx/service_variants) [Python](python/service_variants) [Rust](rust/service_variants)                                                                         | Introduction in service variations, like `local::Service` optimized for intra-process communication or `ipc_threadsafe::Service` where all ports are threadsafe                                                 |
