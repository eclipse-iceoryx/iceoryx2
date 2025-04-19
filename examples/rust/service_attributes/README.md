# Service-Attributes

This example illustrates the definition and usage of custom service attributes.
Service attributes are key-value pairs that can be defined via the service
builder like so:

```rust
node.service_builder(&service_name)
    //...
    .create_with_attributes(
        AttributeSpecifier::new()
            .define("my_custom_key", "some_funky_value")
            .define("my_custom_key", "another_value_for_the_same_key")
            .define("another_key", "another_value")
        )?;
```

When the service is created, the attributes are set. When the service is opened,
these attributes are interpreted as requirements. If a required attribute is not
set, or if its value differs, the service will not be opened. For example, the
following service can be opened because it matches an attribute defined in the
previous example:

```rust
node.service_builder(&service_name)
    //...
    .open_with_attributes(
        AttributeVerifier::new()
            .require("my_custom_key", "some_funky_value")
            // don't care for the value but the key must be present
            .require_key("another_key")
    )?;
```

In contrast, the following example cannot open the service because it requires
an attribute that is not set and another attribute where the value does not
match:

```rust
node.service_builder(&service_name)
    //...
    .open_with_attributes(
        AttributeVerifier::new()
            .require("another_key", "zero")
            .require_key("unknown_key")
    )?;
```

To list all attributes of a service, you can use the following code:

```rust
let service = node.service_builder(&service_name)
    //...
    .open()?;

for attribute in service.attributes().iter() {
    println!("{} = {}", attribute.key(), attribute.value());
}
```

## How to Run

To observe the service attributes in action, open three separate terminals and
execute the following commands:

### Terminal 1

```sh
cargo run --example service_attributes_creator
```

### Terminal 2

```sh
cargo run --example service_attributes_opener
```

### Terminal 3

```sh
cargo run --example service_attributes_incompatible
```

The application in Terminal 3 will fail since it requires incompatible service
attributes.
