Welcome to iceoryx2's C / C++ documentation!
============================================

.. image:: https://user-images.githubusercontent.com/8661268/114321508-64a6b000-9b1b-11eb-95ef-b84c91387cff.png
   :width: 500
   :alt: iceoryx logo

.. toctree::
   :maxdepth: 1

   getting_started

   developer_guide

   iceoryx_hoofs_api

   iceoryx2_c_api

   iceoryx2_cxx_api

   iceoryx2_python_api

Introduction
------------

Welcome to iceoryx2, the efficient, and ultra-low latency inter-process communication
middleware. This library is designed to provide you with fast and reliable
zero-copy and lock-free inter-process communication mechanisms.

So if you want to communicate efficiently between multiple processes or applications
iceoryx2 is for you. With iceoryx2, you can:

* Send huge amounts of data using a publish/subscribe, request/response (planned),
  pipeline (planned) or blackboard pattern (planned),
  making it ideal for scenarios where large datasets need to be shared.
* Exchange signals through events, enabling quick and reliable signaling
  between processes.

iceoryx2 is based on a service-oriented architecture (SOA) and facilitates
seamless inter-process communication (IPC).

It is all about providing a seamless experience for inter-process
communication, featuring versatile messaging patterns. Whether you're diving
into publish-subscribe, events, or the promise of upcoming features like
request-response, pipelines, and blackboard, iceoryx2 has you covered.

One of the features of iceoryx2 is its consistently low transmission latency
regardless of payload size, ensuring a predictable and reliable
communication experience.

iceoryx2's origins can be traced back to `iceoryx <https://github.com/eclipse-iceoryx/iceoryx>`_.
By overcoming past
technical debts and refining the architecture, iceoryx2 enables the modularity
we've always desired.

In the near future, iceoryx2 is poised to support at least the same feature set
and platforms as `iceoryx <https://github.com/eclipse-iceoryx/iceoryx>`_,
ensuring a seamless transition and offering enhanced
capabilities for your inter-process communication needs. So, if you're looking
for lightning-fast, cross-platform communication that doesn't compromise on
performance or modularity, iceoryx2 is your answer.

Performance
-----------

Comparision Of Mechanisms
^^^^^^^^^^^^^^^^^^^^^^^^^

.. image:: ../plots/benchmark_mechanism.svg
   :width: 600
   :alt: benchmark_results

**Benchmark-System**

- **CPU:** AMD Ryzen 7 7840S with Radeon 780M Graphics
- **OS:** Linux 6.8.5-arch1-1 #1 SMP PREEMPT_DYNAMIC GNU/Linux
- **Compiler:**
  - rustc 1.77.1
  - gcc 13.2.1 20230801

Comparision Of Architectures
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. image:: ../plots/benchmark_architecture.svg
   :width: 600
   :alt: benchmark_results

Supported Platforms
-------------------

The support levels can be adjusted when required.

.. list-table:: Supported Platforms
   :widths: 200 100 200 200
   :header-rows: 1

   * - Operating System
     - State
     - Current Support Level
     - Target Support Level
   * - Android
     - planned
     -
     - tier 1
   * - FreeBSD
     - done
     - tier 2
     - tier 1
   * - FreeRTOS
     - planned
     -
     - tier 2
   * - iOS
     - planned
     -
     - tier 2
   * - Linux (x86_64)
     - done
     - tier 2
     - tier 1
   * - Linux (aarch64)
     - done
     - tier 2
     - tier 1
   * - Linux (32-bit)
     - done
     - tier 2
     - tier 1
   * - Mac OS
     - done
     - tier 2
     - tier 2
   * - QNX
     - planned
     -
     - tier 1
   * - VxWorks
     - planned
     -
     - tier 1
   * - WatchOS
     - planned
     -
     - tier 2
   * - Windows
     - done
     - tier 2
     - tier 2

- **tier 1** - All safety and security features are working.
- **tier 2** - Works with a restricted security and safety feature set.
- **tier 3** - Work in progress. Might compile and run or not.

Language Bindings
-----------------

.. list-table:: Language Bindings
   :widths: 200 200
   :header-rows: 1

   * - C / C++
     - done
   * - Python
     - planned
   * - Go
     - planned
   * - C#
     - planned
   * - Lua
     - planned
   * - Java
     - planned
   * - Kotlin
     - planned
   * - Swift
     - planned
   * - Zig
     - planned

Commercial Support
------------------

.. image:: https://github.com/eclipse-iceoryx/iceoryx2/assets/56729169/c3ce8370-6cef-4c31-8259-93ddaa61c43e
   :width: 300
   :alt: ekxide io
   :target: https://ekxide.io

`info@ekxide.io <info@ekxide.io>`_

  * commercial extensions and tooling
  * custom feature development
  * training and consulting
  * integration support
  * engineering services around the iceoryx ecosystem
