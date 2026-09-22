# Benchmarks

1. [Publish-Subscribe](#Publish-Subscribe)
2. [FlatBuffers Publish-Subscribe](#FlatBuffers-Publish-Subscribe)
3. [Request-Response](#Request-Response)
4. [Event](#Event)
5. [Queue](#Queue)
6. [ROS 2 Publish-Subscribe](#ROS-2-Publish-Subscribe)

## Publish-Subscribe

The benchmark quantifies the latency between a `Publisher` sending a message and
a `Subscriber` receiving it. In the setup, a bidirectional connection is
established from process `a` to `b` (service name `a2b`) and back (service name
`b2a`). `Subscriber`s employ multithreaded busy waiting and promptly respond
upon message reception. This process repeats `n` times, and the average latency
is subsequently computed.

```sh
cargo run --bin benchmark-publish-subscribe --release -- --bench-all
```

For more benchmark configuration details, see

```sh
cargo run --bin benchmark-publish-subscribe --release -- --help
```

## FlatBuffers Publish-Subscribe

The benchmark quantifies the latency between a `Publisher` sending a FlatBuffer
message and a `Subscriber` receiving it. The setup is identical to the
[Publish-Subscribe](#Publish-Subscribe) benchmark, but instead of a plain byte
slice the payload is an unbounded FlatBuffer vector that is built with the
FlatBuffers API directly inside the data segment of the `Publisher`. Every
iteration serializes a complete FlatBuffer and acquires its root on the
receiving side, therefore the measured latency also contains the serialization
and deserialization cost.

```sh
cargo run --bin benchmark-publish-subscribe-flatbuffer --release -- --bench-all
```

For more benchmark configuration details, see

```sh
cargo run --bin benchmark-publish-subscribe-flatbuffer --release -- --help
```

### Generate Code and Binary Schema

The generated Rust code and the binary schema are already included in this
benchmark. For completeness, the command used to generate them is documented
below:

```sh
flatc -o benchmarks/publish-subscribe-flatbuffer/src --rust --schema --binary \
    benchmarks/publish-subscribe-flatbuffer/src/payload_data.fbs
```

## Request-Response

The benchmark quantifies two scenarios:

1. The latency between a `Client` sending a request and a `Server` receiving it.
2. The latency of a response stream from an established request-response
   connection, i.e. sending a stream of responses from an `ActiveRequest` to the
   corresponding `PendingResponse`.

In the setup, a bidirectional connection is
established from process `a` to `b` (service name `a2b`) and back (service name
`b2a`). `PendingResponse`s employ multithreaded busy waiting and promptly
respond upon message reception. This process repeats `n` times, and the average
latency is subsequently computed.

```sh
cargo run --bin benchmark-request-response --release
```

For more benchmark configuration details, see

```sh
cargo run --bin benchmark-request-response --release -- --help
```

## Event

The event quantifies the latency between a `Notifier` sending a notification and
a `Listener` waking up from and responding to it. In the setup, a bidirectional
connection is established from process `a` to `b` (service name `a2b`) and back
(service name `b2a`). The `Listener` employs a blocking wait and wakes up on
signal reception to promptly respond with a return signal notification. This
process repeats `n` times, and the average latency is subsequently computed.

```sh
cargo run --bin benchmark-event --release -- --bench-all
```

For more benchmark configuration details, see

```sh
cargo run --bin benchmark-event --release -- --help
```

> [!IMPORTANT]
> When you increase the number of listeners or notifiers beyond a certain limit,
> the benchmark may exceed the per-user file descriptor limit. This limit can be
> increased by adjusting the `nofile` setting in the `/etc/security/limits.conf`
> file:
>
> ```ascii
> *     soft    nofile      4096
> *     hard    nofile      8192
> ```
>
> * `*` – Applies to all users
> * `soft` | `hard` – The soft and hard limits
> * The soft limit is set to 4096, while the hard limit is set to 8192
>
> After making these changes, you can use the following command to increase the
> soft file descriptor limit up to the hard limit:
>
> ```bash
> ulimit -n <new_limit>
> ```

## Queue

The queue quantifies the latency between pushing an element into a queue and
acquiring the element in another thread. In the setup, a bidirectional connection
is established from process `a` to `b` (queue name `queue_a2b`) and back
(queue name `queue_b2a`). The thread that acquires the queue's element
employs a multithreaded busy waiting and promptly respond upon data retrieval.
This process repeats `n` times, and the average latency is subsequently computed.

```sh
cargo run --bin benchmark-queue --release
```

For more benchmark configuration details, see

```sh
cargo run --bin benchmark-queue --release -- --help
```

## ROS 2 Publish-Subscribe

The benchmark quantifies the latency between a `Publisher` sending a message and
a `Subscription` receiving it, with ROS 2 as the middleware. It requires a
sourced ROS 2 environment and therefore lives with the ROS 2 integrations, in
[integrations/ros2/benchmarks/publish-subscribe](../integrations/ros2/benchmarks/publish-subscribe).
