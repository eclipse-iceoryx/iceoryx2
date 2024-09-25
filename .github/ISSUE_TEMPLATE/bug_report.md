---
name: Bug report
about: Create a report to help us improve
title: ''
labels: 'bug'
assignees: ''

---

Before posting the bug, take a look at the
[FAQ](https://github.com/eclipse-iceoryx/iceoryx2/blob/main/FAQ.md)
for a possible solution.

## Required information

**Operating system:**

* OS name, version
* Additionally, on Linux, Mac Os, Unix, output of: `uname -a`
* Additionally, on Windows, output of: `ver`

**Rust version:**
Output of: `rustc --version`

**Cargo version:**
Output of: `cargo --version`

**iceoryx2 version:**
E.g. `v1.2.3` or `main` branch

**Detailed log output:**
Add the call `set_log_level(LogLevel::Trace)` at the beginning of your application
and attach the detailed log-output to the bug ticket.

**Observed result or behaviour:**
A minimalistic running code example that reproduces the bug or
a clear and precise description of the observed result.

**Expected result or behaviour:**
What do you expect to happen?

**Conditions where it occurred / Performed steps:**
Describe how one can reproduce the bug.
