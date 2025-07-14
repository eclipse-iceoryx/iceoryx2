# Service-Attributes

This example illustrates the definition and usage of custom service attributes.
Service attributes are key-value pairs that can be defined via the service
builder like so:

```python
node.service_builder(service_name)
# ...
.create_with_attributes(
    iox2.AttributeSpecifier.new()
        .define(iox2.AttributeKey.new("my_custom_key"),
                iox2.AttributeValue.new("some_funky_value"))
        .define(iox2.AttributeKey.new("my_custom_key"),
                iox2.AttributeValue.new("another_value_for_the_same_key"))
        .define(iox2.AttributeKey.new("another_key"),
                iox2.AttributeValue.new("another_value"))
    )
```

When the service is created, the attributes are set. When the service is opened,
these attributes are interpreted as requirements. If a required attribute is not
set, or if its value differs, the service will not be opened. For example, the
following service can be opened because it matches an attribute defined in the
previous example:

```python
node.service_builder(service_name)
# ...
.open_with_attributes(
    iox2.AttributeVerifier.new()
        .require(iox2.AttributeKey.new("my_custom_key"),
                 iox2.AttributeValue.new("some_funky_value"))
        // don't care for the value but the key must be present
        .require_key(iox2.AttributeKey.new("another_key"))
);
```

In contrast, the following example cannot open the service because it requires
an attribute that is not set and another attribute where the value does not
match:

```python
node.service_builder(service_name)
# ...
.open_with_attributes(
    iox2.AttributeVerifier.new()
        .require(iox2.AttributeKey.new("another_key"),
                 iox2.AttributeValue.new("zero"))
        .require_key(iox2.AttributeKey.new("unknown_key"))
);
```

To list all attributes of a service, you can use the following code:

```python
service = (
    node.service_builder(service_name)
    #...
    .open()
)

for attribute in service.attributes.value:
    print(attribute)
```

To observe the service attributes in action, open three separate terminals and
execute the following commands.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
the detailed instructions in the [Python Examples Readme](../README.md).

First you have to create a python environment, install maturin and compile
iceoryx2 and the language bindings:

```sh
# create python development environment
python -m venv .env

# enter environment
source .env/bin/activate # or source .env/bin/activate.fish

# install maturin
pip install maturin
```

## How to Run

### Terminal 1

```sh
python examples/python/service_attributes/creator.py
```

### Terminal 2

```sh
python examples/python/service_attributes/opener.py
```

### Terminal 3

```sh
python examples/python/service_attributes/incompatible.py
```

The application in Terminal 3 will fail since it requires incompatible service
attributes.
