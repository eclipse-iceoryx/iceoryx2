# Installation Instructions

## Install Dependencies

Since iceoryx2 is written in Rust we need to install that first. We recommend
the [official approach](https://www.rust-lang.org/tools/install).

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Additionally, install poetry to simplify managing the Python virtual
environment:

```sh
curl -sSL https://install.python-poetry.org | python3 -
poetry self add poetry-plugin-shell
```

Then you can set up a virtual environment and install all dependencies using:

```sh
poetry --project iceoryx2-ffi/python install
```

## Build

Compile the iceoryx2 Python language bindings into the virutal
environment by calling:

```sh
poetry --project iceoryx2-ffi/python build-into-venv
```

The language bindings will be then available for use inside the virtual
environment.

## Running Examples

First enter the virtual environment:

```sh
poetry --project iceoryx2-ffi/python shell
```

You can then run any Python example from within the virtual environment:

```sh
python examples/python/publish_subscribe/publisher.py
```
