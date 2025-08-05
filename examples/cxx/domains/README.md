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
> * not use pointers to manage their internal structure
>
> Data types like `std::string` or `std::vector` will cause undefined behavior
> and may result in segmentation faults. We provide alternative data types
> that are compatible with shared memory. See the
> [complex data type example](../complex_data_types) for guidance on how to
> use them.

To achieve this, we create a copy of the global configuration, modify the
setting `config.global.prefix` using the user-provided CLI argument, and then
set up the example accordingly.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C++ Examples Readme](../README.md).

First you have to build the C++ examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON
cmake --build target/ff/cc/build
```

## How to Run

You can experiment with this setup by creating multiple publishers and
subscribers with different service names using `-s $SERVICE_NAME`. Only
publisher-subscriber pairs within the same domain will be able to communicate,
and the discovery tool will only detect services from within the same domain.

### Terminal 1: Subscriber in domain "fuu" subscribing to service "bar"

```sh
./target/ff/cc/build/examples/cxx/domains/example_cxx_domains_subscriber -d "fuu" -s "bar"
```

### Terminal 2: Publisher in domain "fuu" publishing on service "bar"

```sh
./target/ff/cc/build/examples/cxx/domains/example_cxx_domains_publisher -d "fuu" -s "bar"
```

### Terminal 3: List all services of domain "fuu"

```sh
./target/ff/cc/build/examples/cxx/domains/example_cxx_domains_discovery -d "fuu"
```
