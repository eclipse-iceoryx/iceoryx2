# iceoryx2-tunnel

A tunnel for extending `iceoryx2` communication beyond the boundary of a single
host. Payloads and events are propagated by leveraging any kind of protocol or
communication mechanisms.

| Component          | Description                                                                               |
|--------------------|-------------------------------------------------------------------------------------------|
| [tunnel](tunnel)   | Generic tunnel that leverages a backend implementation to propagate payloads and events   |
| [backend](backend) | Traits and types used to implement a backend using some interhost communication mechanism |
| [zenoh](zenoh)     | A backend implementation using Zenoh as the communication mechanism                       |
