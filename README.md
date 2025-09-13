<!-- markdownlint-disable -->

[![CI](https://github.com/eclipse-iceoryx/iceoryx2/workflows/CI/badge.svg)](https://github.com/eclipse-iceoryx/iceoryx2/actions/workflows/build-test.yml?query=branch%3Amain++)
[![Codecov](https://codecov.io/gh/eclipse-iceoryx/iceoryx2/branch/main/graph/badge.svg?branch=main)](https://codecov.io/gh/eclipse-iceoryx/iceoryx2?branch=main)
[![Examples](https://img.shields.io/badge/Examples-gray)](examples/)
[![FAQ](https://img.shields.io/badge/FAQ-gray)](FAQ.md)
[![Gitter](https://badges.gitter.im/eclipse-iceoryx/iceoryx.svg)](https://gitter.im/eclipse/iceoryx)
[![Roadmap](https://img.shields.io/badge/Roadmap-gray)](ROADMAP.md)

<p align="center">
<img src="https://github.com/eclipse-iceoryx/iceoryx2/assets/56729169/3230a125-19e5-4e98-a752-da026a086782" width="50%">
</p>

<!-- markdownlint-enable -->

# iceoryx2 - Zero-Copy Lock-Free IPC with a Rust Core

* [Introduction](#introduction)
* [Performance](#performance)
    * [Comparision Of Mechanisms](#comparision-of-mechanisms)
        * [Benchmark-System](#benchmark-system)
    * [Comparision Of Architectures](#comparision-of-architectures)
* [Documentation](#documentation)
    * [User Documentation](#user-documentation)
    * [Contributor Documentation](#contributor-documentation)
    * [API References](#api-references)
* [Supported Platforms](#supported-platforms)
* [Language Bindings](#language-bindings)
* [Commercial Support](#commercial-support)
* [Thanks To All Contributors](#thanks-to-all-contributors)

## Introduction

Welcome to iceoryx2, the efficient, and ultra-low latency inter-process
communication middleware. This library is designed to provide you with fast and
reliable zero-copy and lock-free inter-process communication mechanisms.

So if you want to communicate efficiently between multiple processes or
applications iceoryx2 is for you. With iceoryx2, you can:

* Send huge amounts of data using a publish/subscribe, request/response,
  pipeline (planned) or blackboard pattern (planned), making it ideal
  for scenarios where large datasets need to be shared.
* Exchange signals through events, enabling quick and reliable signaling between
  processes.

iceoryx2 is based on a service-oriented architecture (SOA) and facilitates
seamless inter-process communication (IPC).

It is all about providing a seamless experience for inter-process communication,
featuring versatile messaging patterns. Whether you're diving into
publish-subscribe, events, request-response, or the promise of upcoming features
like pipelines, and blackboard, iceoryx2 has you covered.

One of the features of iceoryx2 is its consistently low transmission latency
regardless of payload size, ensuring a predictable and reliable communication
experience.

iceoryx2's origins can be traced back to
[iceoryx](https://github.com/eclipse-iceoryx/iceoryx). By overcoming past
technical debts and refining the architecture, iceoryx2 enables the modularity
we've always desired.

In the near future, iceoryx2 is poised to support at least the same feature set
and platforms as [iceoryx](https://github.com/eclipse-iceoryx/iceoryx), ensuring
a seamless transition and offering enhanced capabilities for your inter-process
communication needs. So, if you're looking for lightning-fast, cross-platform
communication that doesn't compromise on performance or modularity, iceoryx2 is
your answer.

## Performance

### Comparision Of Mechanisms

![benchmark of different mechanism](https://raw.githubusercontent.com/eclipse-iceoryx/iceoryx2/refs/heads/main/internal/plots/benchmark_mechanism.svg)

#### Benchmark-System

* **CPU:** Intel i7 13700h
* **OS:** Linux 6.10.10-arch1-1 #1 SMP PREEMPT_DYNAMIC
* **Compiler:**
    * rustc 1.81.0
    * gcc 14.2.1 20240910

### Comparision Of Architectures

![benchmark on different systems](https://raw.githubusercontent.com/eclipse-iceoryx/iceoryx2/refs/heads/main/internal/plots/benchmark_architecture.svg)

## Documentation

### User Documentation

* [The iceoryx2 Book](https://ekxide.github.io/iceoryx2-book) (by [ekxide](https://ekxide.io))
* [Examples](examples)
* [Release Notes](doc/release-notes)
* [User FAQ](FAQ.md)

### Contributor Documentation

* [Contributor FAQ](FAQ_ICEORYX_DEVS.md)

### API References

* [Rust API Reference](https://docs.rs/iceoryx2/latest/iceoryx2/)
* [Python API Reference](https://eclipse-iceoryx.github.io/iceoryx2/python/latest/)
* [C++ API Reference](https://eclipse-iceoryx.github.io/iceoryx2/cxx/latest/)
* [C API Reference](https://eclipse-iceoryx.github.io/iceoryx2/c/latest/)

## Supported Platforms

The support levels can be adjusted when required.

| Operating System | State                    | Current Support Level | Target Support Level |
| ---------------- | :----------------------- | :-------------------: | -------------------: |
| Android          | planned                  |           -           |               tier 1 |
| FreeBSD          | done                     |        tier 2         |               tier 1 |
| FreeRTOS         | planned                  |           -           |               tier 2 |
| ThreadX          | planned                  |           -           |               tier 2 |
| iOS              | planned                  |           -           |               tier 2 |
| Linux (x86_64)   | done                     |        tier 2         |               tier 1 |
| Linux (aarch64)  | done                     |        tier 2         |               tier 1 |
| Linux (32-bit)   | done                     |        tier 2         |               tier 1 |
| Mac OS           | done                     |        tier 2         |               tier 2 |
| QNX 7.1          | done                     |        tier 3         |               tier 1 |
| QNX 8.0          | in-progress              |           -           |               tier 1 |
| VxWorks          | proof-of-concept[^1]     |           -           |               tier 1 |
| WatchOS          | planned                  |           -           |               tier 2 |
| Windows          | done                     |        tier 2         |               tier 2 |

[^1]: A proof-of-concept for VxWorks platform support is available on [this
      branch](https://github.com/ekxide/iceoryx2/blob/vxworks-mvp/doc/development-setup/vxworks.md)
      on the [ekxide](https://ekxide.io) fork

* **tier 1** - All safety and security features are working.
* **tier 2** - Works with a restricted security and safety feature set.
* **tier 3** - Not tested in our CI, so might compile and run or not.

<!-- markdownlint-disable MD027 -->
> [!NOTE]
> Some commercial OS require expensive licenses and the support for these
> platforms rely on funding of the license costs.
<!-- markdownlint-enable MD027 -->

<!-- markdownlint-disable MD027 -->
> [!NOTE]
> Yocto recipes are available at [meta-iceoryx2](https://github.com/eclipse-iceoryx/meta-iceoryx2)
<!-- markdownlint-enable MD027 -->

## Language Bindings

| Language |   State |
| -------- | ------: |
| C / C++  |    done |
| Python   |    done |
| Go       | planned |
| C#       | planned |
| Java     | planned |
| Kotlin   | planned |
| Lua      | planned |
| Swift    | planned |
| Zig      | planned |

## Commercial Support

<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->

<table width="100%">
  <tbody>
    <tr>
      <td align="center" valign="top" width="33%">
        <a href="https://ekxide.io">
        <img src="https://github.com/eclipse-iceoryx/iceoryx2/assets/56729169/c3ce8370-6cef-4c31-8259-93ddaa61c43e" alt="ekxide IO GmbH"/><br />
        </a>
        <a href="mailto:info@ekxide.io">info@ekxide.io</a>
      </td>
      <td>
        <ul>
          <li>commercial extensions and tooling</li>
          <li>custom feature development</li>
          <li>training and consulting</li>
          <li>integration support</li>
          <li>engineering services around the iceoryx ecosystem</li>
        </ul>
      </td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

## Thanks To All Contributors

Thanks to everyone who has contributed to iceoryx2. Without their passion and
dedication, the project wouldn't thrive. A list of people who have committed
code can be found on [github](https://github.com/eclipse-iceoryx/iceoryx2/graphs/contributors).
However, contributions are not limited to code - testing the software, reporting
bugs, and spreading the word about iceoryx2 are all equally valuable. A big
thank you as well to those 'invisible' contributors who play a crucial role
behind the scenes.
