# Benchmarks

1. [Publish-Subscribe](#Publish-Subscribe)
2. [Request-Response](#Request-Response)
3. [Event](#Event)
4. [Queue](#Queue)

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
is established from process `a` to `b` (queue name `queue_a2b`) and back (queue name
`queue_b2a`). The thread that acquires the queue's element employs a multithreaded
busy waiting and promptly respond upon data retrieval. This process repeats `n`
times, and the average latency is subsequently computed.

```sh
cargo run --bin benchmark-queue --release
```

For more benchmark configuration details, see

```sh
cargo run --bin benchmark-queue --release -- --help
```
