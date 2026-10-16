# Simulation Clock

This example demonstrates the clock abstraction in iceoryx2, how a system can be
run under a POSIX clock or a simulated clock that is explicitly controlled by
another process.

The example consists of 4 separate processes:

- `sender` - sends data to the receiver and with every sent sample it notifies
  the `monitor` and the `receiver`
- `receiver` - waits with a timeout of 1s to receive data from the `sender`. If
  the timeout was hit it sends event id 1 to the monitor, if a sample was
  received it sends event id 0 to the monitor.
- `monitor` - monitors with 2 listeners on a waitset if samples are sent,
  received or if a timeout was missed.
- `time controller` - controls the clock and explicitly advances the time
  interactively

## Communication View

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
+---------------+         - - - - - -         |   notifier    | -+
                          :  clock  :         +---------------+
                          - - - - - -



                      +-----------------+
                      | time controller |
                      +-----------------+
```

## Clock View

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
