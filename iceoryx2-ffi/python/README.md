# iceoryx2-ffi-python

## Setup Development Environment

```sh
# install maturin, see
# https://github.com/PyO3/maturin

cd $GIT_ROOT

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

# test python bindings
pytest tests/*

# static code analysis
prospector -m -D -T --with-tool mypy -s veryhigh $GIT_ROOT/examples/python
prospector -m -D -T --with-tool mypy -s veryhigh $GIT_ROOT/iceoryx2-ffi/python/tests

# formatting: import ordering
isort $GIT_ROOT/examples/python
isort $GIT_ROOT/iceoryx2-ffi/python/tests

# formatting
black $GIT_ROOT/examples/python
black $GIT_ROOT/iceoryx2-ffi/python/tests
```
