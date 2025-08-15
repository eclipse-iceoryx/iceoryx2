# Service-Attributes

This example illustrates the definition and usage of custom service attributes.
Service attributes are key-value pairs that can be defined via the attribute
specifier like so:

```c
iox2_attribute_specifier_h attribute_specifier = NULL;
if (iox2_attribute_specifier_new(NULL, &attribute_specifier) != IOX2_OK) {
    // handle error
}

iox2_attribute_specifier_define(&attribute_specifier, "my_custom_key", "some_funky_value");
iox2_attribute_specifier_define(&attribute_specifier, "my_custom_key", "another_value_for_the_same_key");
iox2_attribute_specifier_define(&attribute_specifier, "another_key", "another_value");
```

The attribute specifier is then passed to the
`iox2_service_builder_pub_sub_create_with_attributes` function.

When the service is created, the attributes are set. When the service is opened,
these attributes are interpreted as requirements. If a required attribute is not
set, or if its value differs, the service will not be opened. For example, the
following service can be opened because it matches an attribute defined in the
previous example:

```c
iox2_attribute_verifier_h attribute_verifier = NULL;
if (iox2_attribute_verifier_new(NULL, &attribute_verifier) != IOX2_OK) {
    // handle error
}

iox2_attribute_verifier_require(&attribute_verifier, "my_custom_key", "some_funky_value");
// don't care for the value but the key must be present
iox2_attribute_verifier_require_key(&attribute_verifier, "another_key");

if (iox2_service_builder_pub_sub_open_with_attributes(..., &attribute_verifier, ..., ...) != IOX2_OK) {
    // handle error
}
```

In contrast, the following example cannot open the service because it requires
an attribute that is not set and another attribute where the value does not
match:

```c
// ...

iox2_attribute_verifier_require(&attribute_verifier, "another_key", "zero");
iox2_attribute_verifier_require_key(&attribute_verifier, "unknown_key");

if (iox2_service_builder_pub_sub_open_with_attributes(..., &attribute_verifier, ..., ...) != IOX2_OK) {
    // handle error
}
```

To list all attributes of a service, you can use the following code:

```c
enum {
    AttributeBufferSize = 256
};

// ...

iox2_port_factory_pub_sub_h service = NULL;
if (iox2_service_builder_pub_sub_open(...) != IOX2_OK) {
    // handle error
}

// print attributes
iox2_attribute_set_ptr attribute_set_ptr = iox2_port_factory_pub_sub_attributes(&service);
size_t number_of_attributes = iox2_attribute_set_number_of_attributes(attribute_set_ptr);
printf("Attributes:\n");
for (size_t i = 0; i < number_of_attributes; ++i) {
    iox2_attribute_h_ref attribute_ref = iox2_attribute_set_index(attribute_set_ptr, i);
    char buffer[AttributeBufferSize];
    iox2_attribute_key(attribute_ref, &buffer[0], AttributeBufferSize);
    printf("   Attribute { key: \"%s,", buffer);
    iox2_attribute_value(attribute_ref, &buffer[0], AttributeBufferSize);
    printf(" value: \"%s }\n", buffer);
}
```

To observe the service attributes in action, open three separate terminals and
execute the following commands.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C Examples Readme](../README.md).

First you have to build the C examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON -DBUILD_CXX=OFF
cmake --build target/ff/cc/build
```

## How to Run

### Terminal 1

```sh
./target/ff/cc/build/examples/c/service_attributes/example_c_service_attributes_creator
```

### Terminal 2

```sh
./target/ff/cc/build/examples/c/service_attributes/example_c_service_attributes_opener
```

### Terminal 3

```sh
./target/ff/cc/build/examples/c/service_attributes/example_c_service_attributes_incompatible
```

The application in Terminal 3 will fail since it requires incompatible service
attributes.
