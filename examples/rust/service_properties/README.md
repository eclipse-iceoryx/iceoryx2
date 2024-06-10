# Service-Properties

## Running the Example

This example illustrates the definition and usage of custom service properties. Properties are key-value pairs that can be defined via the service builder like so:

```rust
zero_copy::Service::new(&service_name)
    .add_property("my_custom_key", "some_funky_value")
    .add_property("my_custom_key", "another_value_for_the_same_key")
    .add_property("another_key", "another_value")
    //...
    .create()?;
```

When the service is created, the properties are set. When the service is opened, these properties are interpreted as requirements. If a required property is not set, or if its value differs, the service will not be opened. For example, the following service can be opened because it matches a property defined in the previous example:

```rust
zero_copy::Service::new(&service_name)
    .add_property("my_custom_key", "some_funky_value")
    //...
    .open()?;
```

In contrast, the following example cannot open the service because it requires a property that is not set and another property where the value does not match:

```rust
zero_copy::Service::new(&service_name)
    .add_property("unknown_key", "whatever")
    .add_property("another_key", "zero")
    //...
    .open()?;
```

To list all properties of a service, you can use the following code:

```rust
let service = zero_copy::Service::new(&service_name)
    //...
    .open()?;

for property in service.properties().iter() {
    println!("{} = {}", property.key(), property.value());
}
```

To observe the service properties in action, open three separate terminals and execute the following commands:

**Terminal 1**

```sh
cargo run --example service_properties_creator
```

**Terminal 2**

```sh
cargo run --example service_properties_opener
```

**Terminal 3**

```sh
cargo run --example service_properties_incompatible
```

The application in Terminal 3 will fail since it requires incompatible service properties.
