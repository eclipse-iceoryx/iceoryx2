[![Benchmarks](https://img.shields.io/badge/Benchmarks-gray)](benchmarks/README.md)
[![Best Practices](https://img.shields.io/badge/Best_Practices-gray)](BEST_PRACTICES.md)
[![Changelog](https://img.shields.io/badge/Changelog-gray)](CHANGELOG.md)
[![Contributing](https://img.shields.io/badge/Contributing-gray)](CONTRIBUTING.md)
[![Examples](https://img.shields.io/badge/Examples-gray)](examples/)
[![FAQ](https://img.shields.io/badge/FAQ-gray)](FAQ.md)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Roadmap](https://img.shields.io/badge/Roadmap-gray)](ROADMAP.md)

# iceoryx2 - Zero-Copy Lock-Free IPC Purely Written In Rust

 1. [Introduction](#introduction)
 2. [Performance](#performance)
 3. [Getting Started](#getting-started)
    1. [Publish Subscribe](#publish-subscribe)
    2. [Events](#events)
    3. [Custom Configuration](#custom-configuration)
 4. [Supported Platforms](#supported-platforms)
 5. [Language Bindings](#language-bindings)
 6. [Thanks To All Contributors](#thanks-to-all-contributors)

## Introduction

Welcome to Iceoryx2, the efficient, and ultra-low latency inter-process communication
middleware. This library is designed to provide you with fast and reliable
zero-copy and lock-free inter-process communication mechanisms.

Iceoryx2 is all about providing a seamless experience for inter-process
communication, featuring versatile messaging patterns. Whether you're diving
into publish-subscribe, events, or the promise of upcoming features like
request-response, pipelines, and blackboard, Iceoryx2 has you covered.

One of the features of Iceoryx2 is its consistently low transmission latency
regardless of payload size, ensuring a predictable and reliable
communication experience.

Iceoryx2's origins can be traced back to
[iceoryx](https://github.com/eclipse-iceoryx/iceoryx). By overcoming past
technical debts and refining the architecture, Iceoryx2 enables the modularity
we've always desired.

In the near future, Iceoryx2 is poised to support at least the same feature set
and platforms as [iceoryx](https://github.com/eclipse-iceoryx/iceoryx),
ensuring a seamless transition and offering enhanced
capabilities for your inter-process communication needs. So, if you're looking
for lightning-fast, cross-platform communication that doesn't compromise on
performance or modularity, Iceoryx2 is your answer.

## Performance

```mermaid
gantt
    title Latency (in ns) - 64b payload
    dateFormat X
    axisFormat %s

    section iceoryx2
    240 : 0, 240
    section iceoryx
    1000 : 0, 1000
    section MQueue
    700 : 0, 700
    section UDS
    1500 : 0, 1500
```

```mermaid
gantt
    title Latency (in ns) - 64kb payload
    dateFormat X
    axisFormat %s

    section iceoryx2
    240 : 0, 240
    section iceoryx
    1000 : 0, 1000
    section MQueue
    14000 : 0, 14000
    section UDS
    23000 : 0, 23000
```

**Benchmark-System**

- **CPU:** Intel(R) Core(TM) i7-10875H CPU @ 2.30GHz
- **OS:** Linux 6.5.9-arch2-1 #1 SMP PREEMPT_DYNAMIC GNU/Linux
- **Compiler:**
  - rustc 1.72.1
  - gcc 13.2.1 20230801

## Getting Started

### Publish Subscribe

This minimal example showcases a publisher sending the number 1234 every second,
while a subscriber efficiently receives and prints the data.

**publisher.rs**

```rust
use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service_name = ServiceName::new("My/Funk/ServiceName")?;

    let service = zero_copy::Service::new(&service_name)
        .publish_subscribe()
        .open_or_create::<usize>()?;

    let publisher = service.publisher().create()?;

    while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
        let sample = publisher.loan_uninit()?;
        let sample = sample.write_payload(1234);
        publisher.send(sample)?;
    }

    Ok(())
}
```

**subscriber.rs**

```rust
use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service_name = ServiceName::new("My/Funk/ServiceName")?;

    let service = zero_copy::Service::new(&service_name)
        .publish_subscribe()
        .open_or_create::<usize>()?;

    let subscriber = service.subscriber().create()?;

    while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
        while let Some(sample) = subscriber.receive()? {
            println!("received: {:?}", *sample);
        }
    }

    Ok(())
}
```

This example is a simplified version of the
[publish-subscribe example](examples/examples/publish_subscribe/). You can
execute it by opening two terminals and calling:

**Terminal 1:**

```sh
cargo run --example publish_subscribe_publisher
```

**Terminal 2:**

```sh
cargo run --example publish_subscribe_subscriber
```

### Events

This minimal example showcases an event notification between two processes.

**notifier.rs**

```rust
use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_name = ServiceName::new("MyEventName")?;

    let event = zero_copy::Service::new(&event_name)
        .event()
        .open_or_create()?;

    let notifier = event.notifier().create()?;

    let mut counter: u64 = 0;
    while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
        counter += 1;
        notifier.notify_with_custom_event_id(EventId::new(counter))?;

        println!("Trigger event with id {} ...", counter);
    }

    Ok(())
}
```

**listener.rs**

```rust
use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_name = ServiceName::new("MyEventName")?;

    let event = zero_copy::Service::new(&event_name)
        .event()
        .open_or_create()?;

    let mut listener = event.listener().create()?;

    while let Iox2Event::Tick = Iox2::wait(Duration::ZERO) {
        if let Ok(events) = listener.timed_wait(CYCLE_TIME) {
            for event_id in events {
                println!("event was triggered with id: {:?}", event_id);
            }
        }
    }

    Ok(())
}
```

This example is a simplified version of the
[event example](examples/examples/event/). You can
execute it by opening two terminals and calling:

**Terminal 1:**

```sh
cargo run --example event_notifier
```

**Terminal 2:**

```sh
cargo run --example event_listener
```

### Custom Configuration

It is possible to configure default quality of service settings, paths and file suffixes in a
custom configuration file. For more details visit the [configuration directory](config/).

## Supported Platforms

The support levels can be adjusted when required.

| Operating System | State        | Current Support Level | Target Support Level |
|------------------|:-------------|:---------------------:|---------------------:|
| Android          | planned      | -                     | tier 1               |
| FreeBSD          | done         | tier 2                | tier 1               |
| FreeRTOS         | planned      | -                     | tier 2               |
| iOS              | planned      | -                     | tier 2               |
| Linux (x86_64)   | done         | tier 2                | tier 1               |
| Linux (aarch64)  | done         | tier 2                | tier 1               |
| Linux (32-bit)   | in-progress  | tier 3                | tier 1               |
| Mac OS           | in-progress  | tier 3                | tier 2               |
| QNX              | planned      | -                     | tier 1               |
| WatchOS          | planned      | -                     | tier 2               |
| Windows          | done         | tier 2                | tier 2               |

- **tier 1** - All safety and security features are working.
- **tier 2** - Works with a restricted security and safety feature set.
- **tier 3** - Work in progress. Might compile and run or not.

## Language Bindings

| Language | State    |
|----------|---------:|
| C / C++  | planned  |
| Lua      | planned  |
| Python   | planned  |
| Zig      | planned  |

## Thanks To All Contributors

<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->

<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%">
          <a href="https://github.com/elfenpiff">
          <img src="https://avatars.githubusercontent.com/u/56729169" width="120px;" alt="Christian »elfenpiff« Eltzschig"/><br />
          <sub><b>Christian »elfenpiff« Eltzschig</b></sub></a></td>
      <td align="center" valign="top" width="14.28%">
          <a href="https://github.com/elboberido">
          <img src="https://avatars.githubusercontent.com/u/56729607" width="120px;" alt="Mathias »elBoberido« Kraus"/><br />
          <sub><b>Mathias »elBoberido« Kraus</b></sub></a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->
