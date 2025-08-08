# Domains

Let's assume you want to create multiple iceoryx2 groups of processes where the
processes inside a group can communicate and interact with each other. However,
the groups themselves should remain isolated, meaning a process from one group
cannot interact with a process from another group.

In other words, we aim to create different iceoryx2 domains on a local machine
that are strictly separated.

This strict separation can be achieved by using the iceoryx2 configuration.
Within the configuration, a wide range of parameters can be adjusted, such as
the directory used for files containing static service information (a detailed
description of the service) or static node information (a detailed description
of a node). Additionally, the prefix of all files, which is by default `iox2_`,
can be modified.

In this example, we use the prefix to separate the iceoryx2 groups. For all
examples, the user can set the iceoryx2 domain using `-d $DOMAIN_NAME$`. The
domain name must be a valid file name. The example will only operate within this
domain and cannot interact with any services in other domains with different
names.

The `domains_discovery` binary illustrates this by listing all services
available in a given domain. Similarly, the `domains_publisher` will send data
only to subscribers within the same domain. Subscribers in other domains will
not receive any data.

## Implementation

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

To achieve this, we create a copy of the global configuration, modify the
setting `config.global.prefix` using the user-provided CLI argument, and then
set up the example accordingly.

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
poetry --project iceoryx2-ffi/python run maturin develop --manifest-path iceoryx2-ffi/python/Cargo.toml --target-dir target/ffi/python
```

## How to Run

You can experiment with this setup by creating multiple publishers and
subscribers with different service names by using the second argument. Only
publisher-subscriber pairs within the same domain will be able to communicate,
and the discovery tool will only detect services from within the same domain.

### Terminal 1: Subscriber in domain "fuu" subscribing to service "bar"

```sh
poetry --project iceoryx2-ffi/python run python examples/python/domains/subscriber.py fuu bar
```

### Terminal 2: Publisher in domain "fuu" publishing on service "bar"

```sh
poetry --project iceoryx2-ffi/python run python examples/python/domains/publisher.py fuu bar
```

### Terminal 3: List all services of domain "fuu"

```sh
poetry --project iceoryx2-ffi/python run python examples/python/domains/discovery.py fuu
```
