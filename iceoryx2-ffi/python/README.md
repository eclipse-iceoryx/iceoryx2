# iceoryx2-ffi-python

## Running Examples

```sh
export PYTHONPATH="$(git rev-parse --show-toplevel)/iceoryx2-ffi/python/python-src"
python examples/python/event/listener.py
```

## Setup Development Environment

```sh
# install maturin, see
# https://github.com/PyO3/maturin

cd $(git rev-parse --show-toplevel)

# create python development environment
python -m venv .env

# enter environment
source .env/bin/activate # or source .env/bin/activate.fish

# install dependencies
pip install pytest
pip install prospector[with_mypy]
pip install black
pip install isort
```

## Development

```sh
# compile PyO3 bindings
cd iceoryx2-ffi/python
maturin develop

export PYTHONPATH="$(git rev-parse --show-toplevel)/iceoryx2-ffi/python/python-src"
# test python bindings
pytest tests/*

cd $(git rev-parse --show-toplevel)

# static code analysis
prospector -m -D -T --with-tool mypy -s veryhigh $(pwd)/examples/python
prospector -m -D -T --with-tool mypy -s veryhigh $(pwd)/iceoryx2-ffi/python/tests

# formatting: import ordering
isort $(pwd)/examples/python
isort $(pwd)/iceoryx2-ffi/python/tests

# formatting
black $(pwd)/examples/python
black $(pwd)/iceoryx2-ffi/python/tests
```
