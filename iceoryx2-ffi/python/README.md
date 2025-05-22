## Setup

```
# first time creating env
python -m venv .env

# enter env
cd iceoryx2-ffi/python
bash
source .env/bin/activate

# first time entering env
pip install pytest
pip install prospector[with_mypy]
```

## Development

```
maturin develop # compile PyO3 bindings
```

## Testing

```
pytest iceoryx2-ffi/python/tests/*
```

## Static Code Analysis

```
cd iceoryx2-ffi/python
prospector -m -D -T --with-tool mypy -s veryhigh
```

## ToDo

* configure `pyproject.toml`
