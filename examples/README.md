# Examples

## Foundations of Communication in iceoryx2 Applications

In the world of iceoryx2, communication is at the core of everything. To
facilitate seamless communication, we always begin by opening or creating a
service, the fundamental building block of an iceoryx2 application. Services
serve as the conduit through which different parts of your application
interact.

The type of service you create is defined by a messaging pattern, which
dictates how data flows between participants. iceoryx2 supports various
messaging patterns, including:

* **Publish-Subscribe:** In this pattern, a publisher sends a continuous stream
    of data to one or more subscribers, enabling real-time data dissemination.

* **Request-Response:** (planned) This pattern enables clients to send requests
    to a server, which responds with the requested data or action,
    making it suitable for interactive, transactional communication.

* **Pipeline:** (planned) Borrowed from the Unix command line, this pattern
    involves a data source that produces data and transfers ownership to a
    sink, where it can be modified or processed in a pipeline-like fashion.

The service acts as a factory, creating service participants, often called
"ports." These ports establish communication links between
different components of your application. Each service can be customized with
specific settings to meet your application's requirements.

Within the service builder, you can configure various service-specific
quality of service parameters, ensuring that your communication behaves
precisely as you intend. The service port factory allows you to fine-tune the
settings and behavior of individual ports, giving you precise control over how
they interact and exchange data.

## Overview

| Name | Description |
|------|-------------|
| [complex data types](rust/complex_data_types) | Send zero-copy compatible versions of `Vec` and `String`. Introduces `PlacementDefault` trait for large data types to perform an in place initialization where otherwise a stack overflow would be encountered.|
| [discovery](rust/discovery) | List all available services in a system. |
| [docker](rust/docker) | Communicate between different docker containers and the host. |
| [event](rust/event) | Exchanging event signals between multiple processes.|
| [publish subscribe](rust/publish_subscribe) | Communication between multiple processes with a [publish subscribe messaging pattern](https://en.wikipedia.org/wiki/Publish–subscribe_pattern). |
| [publish subscribe dynamic data](rust/publish_subscribe_dynamic_data) | Communication between multiple processes with a [publish subscribe messaging pattern](https://en.wikipedia.org/wiki/Publish–subscribe_pattern) and payload data that has a dynamic size. |
| [service attributes](rust/service_attributes) | Creates a service with custom attributes that are available to every endpoint. If the attributes are not compatible the service will not open. |
