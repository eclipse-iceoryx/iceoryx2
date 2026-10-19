# Simulation Clock

This example demonstrates the clock abstraction in iceoryx2 and shows how a
system can run using either a POSIX clock or a simulated clock that is
explicitly controlled by another process.

The example consists of three communicating processes:

* **`sender`** - Sends a sample to the receiver every second and notifies the
  monitor whenever a sample is sent.
* **`receiver`** - Waits up to 2 seconds to receive a sample from the sender. It
  notifies the monitor with event ID `0` when a sample is received or event ID
  `1` when the timeout expires.
* **`monitor`** - Uses a waitset with two listeners to monitor sent and received
  samples, including receiver timeouts.

```ascii
                +-----------------------------+
                | monitor                     |
                +- - - - - - - - - - - - - - -+
                |      listener (sample recv) | <----------------+
                |  +-> listener (sample sent) |                  |
                +--+--------------------------+                  |
                   |                                             |
+---------------+  |                          +---------------+  |
| sender        |  |                          | receiver      |  |
+- - - - - - - -+  |                          +- - - - - - - -+  |
|   notifier    | -+------------------------> |   listener    |  |
|   publisher   | --------------------------> |   subscriber  |  |
+---------------+                             |   notifier    | -+
                                              +---------------+
```

A fourth process, the **`time controller`**, controls the simulated clock.
Instead of relying on the passage of real time, it allows the user to advance
the clock explicitly.

```ascii
                 +---------------------------+
                 | monitor                   |
                 +- - - - - - - - - - - - - -+
                 |   listener (sample sent)  |
                 |   listener (sample recv)  |
                 +---------------------------+
                               ^
+---------------+              |              +---------------+
| sender        |              |              | receiver      |
+- - - - - - - -+              |              +- - - - - - - -+
|   notifier    |              |              |   listener    |
|   publisher   |              |              |   subscriber  |
+---------------+ <------ - - - - - - ------> |   notifier    |
                          :  clock  :         +---------------+
                          - - - - - -
                               ^
                            control
                               |
                      +-----------------+
                      | time controller |
                      +-----------------+
```

## How to Run

Open four separate terminals and start one process in each.

### Terminal 1: Sender

```sh
# simulation clock
cargo run --example simulation_clock_sender sim

# real clock
cargo run --example simulation_clock_sender real
```

### Terminal 2: Receiver

```sh
# simulation clock
cargo run --example simulation_clock_receiver sim

# real clock
cargo run --example simulation_clock_receiver real
```

### Terminal 3: Monitor

```sh
# simulation clock
cargo run --example simulation_clock_monitor sim

# real clock
cargo run --example simulation_clock_monitor real
```

### Terminal 4: Time Controller

```sh
cargo run --example simulation_clock_time_controller
```

> [!CAUTION]
> When the time controller is not stopped by entering `e` or `q`, the
> underlying resources will remain and cannot be reopened by the time
> controller. In this case, you need to stop all processes, remove all iceoryx2
>> resources with
> `sudo rm -rf /dev/shm/iox2_*; sudo rm -rf /tmp/iceoryx2` and restart the
> example.
> This is only a temporary workaround until the feature #2100 is finalized.

The sender waits one second between samples and notifies the monitor whenever a
sample is sent. The receiver waits up to two seconds for the next sample. If no
sample arrives within that time, it reports a timeout. The monitor observes the
notifications from both processes and reports whether samples were sent,
received, or missed.

Using the time controller, you can advance the simulated clock in discrete
steps. For example, advancing the clock by one hour causes processes waiting for
a one-second timeout to wake up, as their deadlines have long since passed.

From the sender's perspective, which is supposed to send a sample every second,
this looks as if the operating system had not scheduled it for an entire hour.

While this is an extreme example, it illustrates a real-world scenario: under
heavy system load, the operating system may be unable to schedule every process
on time. As a result, processes can wake up significantly later than expected.

The simulated clock makes it possible to reproduce such timing scenarios in a
controlled and predictable way, without having to create the corresponding
system load or wait for real time to pass.
