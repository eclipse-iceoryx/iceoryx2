# Python Bindings Development Environment

Quick reference for commands relevant for development of the `iceoryx2` Python bindings.

## Install Poetry

```sh
curl -sSL https://install.python-poetry.org | python3 -
# ensure that ~/.local/bin is in the PATH environment variable
which poetry # should show '$HOME/.local/bin/poetry'
poetry self add poetry-plugin-shell
```

## Setup Virtual Environment

```sh
cd $(git rev-parse --show-toplevel)

# Install dependencies and create virtual environment
poetry --project iceoryx2-ffi/python install

# (OPTIONAL) Enter the virtual environment - skip the 'poetry run' prefix for all commands
poetry --project iceoryx2-ffi/python shell
```

## Development

```sh
cd $(git rev-parse --show-toplevel)

# Compile PyO3 bindings
poetry --project iceoryx2-ffi/python build-into-venv

# Test Python bindings
poetry --project iceoryx2-ffi/python test

# Run static code analysis
poetry --project iceoryx2-ffi/python check-linting
poetry --project iceoryx2-ffi/python check-imports
poetry --project iceoryx2-ffi/python check-formatting

# Fix some issues automatically
poetry --project iceoryx2-ffi/python fix-imports
poetry --project iceoryx2-ffi/python fix-formatting
```

### Security Issue: Update Dependency

```sh
poetry --project iceoryx2-ffi/python update urllib3
poetry --project doc/api/python update urllib3
```

## Run Examples

```sh
cd $(git rev-parse --show-toplevel)

poetry --project iceoryx2-ffi/python run python examples/python/event/listener.py
```

## Notifier Callback Synchronization

The Python notifier uses a private admission state before locking its Rust
value. Admission is idle, owned by an ordinary operation, or owned by a listener
traversal. Ordinary operations from another thread wait on a condition variable
while detached from Python. Same-thread re-entry is rejected. Traversal admission
rejects every competing operation, including property reads and deletion, so a
callback can join a thread that attempts access without creating a wait cycle.

The admission mutex is never held while acquiring the Rust value lock or calling
Python. The value lock is released before the admission permit. Starting or
ending a traversal wakes waiters to recheck the state. This preserves ordinary
serialization without the race of checking an independent flag before blocking
on the value mutex. A rejected delete leaves the notifier intact.

Each callback gets a separate Monofier state. Only the creating thread can use
its erased pointer, and only until that callback frame exits. A private RAII
guard permanently invalidates the pointer; normal return invalidates it before
extracting or destroying the Python result. Pointer and Service-specific thunk
are paired at the single construction site. The bridge reconstructs only a
short-lived shared Rust reference and calls the original Rust Monofier directly.
It never stores a fabricated static reference or re-enters the Python notifier.

The raw-pointer bridge is isolated in `notifier_callback.rs`. Its safety relies
on the creating thread's synchronous call stack, not just an atomic validity
flag or the GIL. Changes to thread dispatch, asynchronous callbacks or ownership
must re-evaluate that invariant. Callback errors are retained as `PyErr`, stop
Rust traversal, and reach Python after both locks and admission are released.

Callback contract tests run in child processes with hard timeouts, including a
logging callback that holds an ordinary operation while a second thread waits.
This checks GIL handling as well as contention, without requiring a free-threaded
interpreter. Rust unit tests also exercise the admission state independently.
