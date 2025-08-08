# iceoryx2-ffi-python

Quick reference for commands relevant for development of the `iceoryx2` Python bindings.

## Install Poetry

```sh
curl -sSL https://install.python-poetry.org | python3 -
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

# Test python bindings
poetry --project iceoryx2-ffi/python test

# Run static code analysis
poetry --project iceoryx2-ffi/python check-linting
poetry --project iceoryx2-ffi/python check-imports
poetry --project iceoryx2-ffi/python check-formatting
```

## Run Examples

```sh
cd $(git rev-parse --show-toplevel)

poetry --project iceoryx2-ffi/python run python examples/python/event/listener.py
```
