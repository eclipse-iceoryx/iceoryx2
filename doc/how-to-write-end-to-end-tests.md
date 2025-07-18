# End-to-End Testing with `expect`

This guide shows how to write **end-to-end tests** for iceoryx2 using `expect`.

`expect` is a powerful automation tool designed to interact with processes. It
is often used for tasks like testing command-line applications, where one needs
to handle prompts or check output. The tool can spawn processes, wait for
specific outputs, and respond with predefined inputs, making it especially
useful for end-to-end testing scenarios. `expect` works by defining patterns to
match against process outputs and can handle situations like timeouts or
unexpected EOFs, providing robust error handling and automation.

The convention in iceoryx2 is to have `test_e2e_foo.exp` files for the tests.
These files will be automatically found and executed in the CI.

See following links for more information regarding `expect`:
<!-- markdownlint-disable MD034 Bare URL used -->
* https://phoenixnap.com/kb/linux-expect
* https://draculaservers.com/tutorials/linux-expect-command
<!-- markdownlint-enable MD034 Bare URL used -->

## Common Setup

```sh
#!/usr/bin/expect

set REPO_ROOT [exec git rev-parse --show-toplevel]
cd ${REPO_ROOT}

source examples/cross-language-end-to-end-tests/common.exp
```

Here, `git rev-parse --show-toplevel` is used to find the root directory of the
repository and change into that directory to ensure that all file paths in the
tests are correct, regardless of the location where the script is run.

Furthermore, a `common.exp` file is sourced to provide access to commonly used
constants and functions.

The `common.exp` file contains constants for color output, such as `C_RED`,
`C_GREEN`, `C_YELLOW`, `C_BLUE`, and `C_OFF`. It can be used like this:

```sh
puts "${C_GREEN}All glory to the hypnotoad!${C_OFF}"
```

The `common.exp` file also contains the following functions:

* `expect_output`: A simple check for the output of the most recently spawned
  process. `*` can be used as a wildcard, and `\` to escape special characters
  like `"`.
* `expect_output_from`: Used when checking the output of a process that is not
  the most recently spawned. The first parameter of this function is the
  `spawn_id` of the process to check for output.
* `show_test_passed`: Should be placed at the end of the `test_e2e_*.exp` file
  to indicate a successful test run.

It also includes the following helper functions:

* `handle_timeout`: Prints an error message and aborts the test when the
  defined timeout expires.
* `handle_end_of_file`: Prints an error message and aborts the test when an
  `eof` is encountered instead of the expected string. This usually happens when
  an application shuts down and the expected string was not printed.

## Minimal Example

The following is the most basic example, where we spawn two processes and assert
that the most recently spawned one outputs the expected string.

```sh
#!/usr/bin/expect

#### Common Setup

set REPO_ROOT [exec git rev-parse --show-toplevel]
cd ${REPO_ROOT}

source examples/cross-language-end-to-end-tests/common.exp

#### Test Setup

set timeout 10

spawn cargo run --example publish_subscribe_publisher

spawn cargo run --example publish_subscribe_subscriber

#### Test Assertion

expect_output "received: TransmissionData { x: 3, y: 9, funky: 2436.36 }*"

show_test_passed
```

In this setup:

* `set timeout 10` sets the maximum execution time for the script to 10
  seconds.
* `spawn` is used to start each of the processes we want to test.
* `expect_output` waits for the provided output string from the most
  recently spawned process.
* `show_test_passed` is used to indicate a successful test.

## Complex Example

Now let’s look at a more complex example, where several processes are spawned
and interacted with. In this case, the outputs of multiple services are checked
(including those of processes that are not the most recently spawned), and
certain processes are terminated.

```sh
#!/usr/bin/expect

#### Common Setup

set REPO_ROOT [exec git rev-parse --show-toplevel]
cd ${REPO_ROOT}

source examples/cross-language-end-to-end-tests/common.exp

#### Test Setup and Assertion

set timeout 20

spawn cargo run --example health_monitoring_central_daemon
set id_daemon $spawn_id
# wait for daemon to be ready
expect_output "Central daemon up and running."

spawn cargo run --example health_monitoring_publisher_1
set id_publisher_1 $spawn_id

spawn cargo run --example health_monitoring_publisher_2
set id_publisher_2 $spawn_id

spawn cargo run --example health_monitoring_subscriber
set id_subscriber $spawn_id

expect_output_from $id_publisher_1 "service_1: Send sample*"
expect_output_from $id_publisher_2 "service_2: Send sample*"
expect_output_from $id_subscriber "service_1: Received sample*"

exec kill -SIGKILL [exp_pid -i $id_publisher_1]

expect_output_from $id_daemon "detected dead node"

exec kill -SIGKILL [exp_pid -i $id_daemon]
exec kill -SIGKILL [exp_pid -i $id_publisher_2]

expect_output_from $id_subscriber "detected dead node"

show_test_passed
```

In this setup:

* `set id_foo $spawn_id` is used to store the `spawn_id` of the currently
  spawned process for later use.
* `expect_output` right after `spawn` is used to wait for the spawned process
  to become operational before the next one is spawned.
* `exec kill -SIGKILL [exp_pid -i $id_foo]` sends a `SIGKILL` signal to the
  process with the provided `spawn_id`. Alternatively, `-SIGINT` and `-SIGTERM`
  can also be used.
* `expect_output_from $id_foo "bar"` checks for the output of the process with
  the provided `spawn_id`. To permanently use the output of a specific process
  for the checks, `set spawn_id $id_foo` can be used.

<!-- markdownlint-disable MD082 Blank line inside blockquote -->
> [!NOTE]
> `expect_output` and `expect_output_from` can be used as synchronization
> points, e.g. if an application needs to reach a specific operational state
> before other applications can be started.

> [!NOTE]
> When multiple processes are spawned, it might happen that `expect` completely
> stops the processes spawned first. In this case, the `expect_output` will run
> in a timeout. To make `expect` run the initially spawned processes, it is
> sufficient to have a `expect_output_from` call on the `spawn_id` of the
> respective process before checking the output of the actual process.
<!-- markdownlint-enable MD082 -->
