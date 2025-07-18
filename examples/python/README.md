# Installation Instructions

## Install Dependencies

Since iceoryx2 is written in Rust we need to install that first. We recommend
the [official approach](https://www.rust-lang.org/tools/install).

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

You need to have at least python 3.9 installed. Create a new environment with

```sh
cd iceoryx2
python -m venv .env
```

and enter it by calling

```sh
source .env/bin/activate        # for bash shell
source .env/bin/activate.fish   # for fish shell
```

Then some dependencies must be installed with `pip`

```sh
pip install maturin
```

### Install Development Dependencies

If you would like to run the iceoryx2 python unit tests you need to install
`pytest` additionally.

```sh
pip install pytest
```

If you would like to start developing on the python bindings you need to
install:

```sh
pip install prospector[with_mypy]
pip install black
pip install isort
pip install bandit
```

## Build

Compile iceoryx2 and the python language bindings by calling

```sh
maturin develop --manifest-path iceoryx2-ffi/python/Cargo.toml
```

## Running Example

To run any python example, please ensure that you have created an environment
and entered it first.

```sh
cd iceoryx2
python -m venv .env # creates the new environment, needs to be called only once

source .env/bin/activate        # for bash shell
source .env/bin/activate.fish   # for fish shell
```

Then start the example by calling:

```sh
cd iceoryx2
python examples/python/publish_subscribe/publisher.py
```
