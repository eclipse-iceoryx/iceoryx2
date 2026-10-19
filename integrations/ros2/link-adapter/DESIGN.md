# Design Notes

Notable details on the design of the ROS 2 adapter for the `iceoryx2-link`.

## Discovery

The adapter lists the topics and services via the ROS graph.

* A topic is listed when a node other than the gateway's publishes or
  subscribes to it, with the QoS the graph reports for the first such
  endpoint. A topic with several types is skipped.
* A service is listed when a node other than the gateway's serves or
  calls it. The nodes are walked one by one, and each is asked for its
  servers and clients.
* Internal endpoints are skipped: the `/rosout` and `/parameter_events`
  topics, the parameter services every node serves, and the type
  description service nodes serve from Iron on.
* Every service is listed with the default QoS profile.
* iceoryx2 events have no ROS 2 counterpart and are not propagated.

### Caveats

#### Services on the graph

The graph tells less about services than about topics:

|                         | topics                         | services                                |
|-------------------------|--------------------------------|-----------------------------------------|
| matched counts          | per publisher and subscription | none, a client can ask for a server     |
| QoS of remote endpoints | available                      | none before Lyrical                     |

`rcl_count_services` and `rcl_count_clients` exist from Jazzy on. Humble
and Iron have only the per-node queries, which is why listing must check
the services of each node itself.

#### Service QoS

A service's QoS is not checked, and the gateway opens its client and
server on a QoS profile it cannot confirm. A ROS 2 service on an
incompatible QoS profile never matches them, without an error, so the
QoS profile the mapping yields for a service has to be compatible with
the one its ROS 2 endpoints use.

* The static mapping accepts a service discovered on the graph when it
  is present in the configuration, and opens the gateway's endpoints
  with the QoS the entry specifies. Conversely, the QoS the graph reports
  for a topic is compared with its entry's and a mismatch refused.
* The prefix mapping accepts any service and opens the gateway's
  endpoints with the default QoS profile of ROS 2 services. Requests to
  a ROS 2 service on a QoS profile incompatible with it are silently
  lost.

## Publish-Subscribe

A topic maps to one iceoryx2 publish-subscribe service. The gateway holds
a publisher and a subscription on the topic and a subscriber and a
publisher on the service, and relays between them.

`rcl` publishes and takes topic messages in their serialized form, as the
CDR bytes the rmw puts on the wire. The adapter moves those bytes as they
are:

```text
send:  shared memory ──translator──▶ CDR ──rcl_publish_serialized_message──▶ wire
take:  shared memory ◀──translator── CDR ◀──rcl_take_serialized_message──── wire
```

* The translator converts between the sample a local service carries and
  CDR. The passthrough translator has nothing to convert, the plain-struct
  translator transcodes the C struct.
* A take writes the message straight into a loaned sample. The rmw asks
  for the buffer once it knows the size, and the adapter answers with the
  sample's payload.
* The message info of a take, the publisher's GID, its timestamp and
  sequence number, is offered to the local service as the `RosHeader`
  user header.

### Caveats

#### Loopback

`rcl` delivers a publication to every subscription on a topic, the
gateway's own included:

```text
gateway publisher ──message──▶ ROS 2 subscription
                          └──▶ gateway subscription   (skipped)
```

The gateway's subscription skips the publications of the gateway's own
publisher by the GID the message info carries. `ignore_local_publications`
is not used for this. With it set, `rmw_fastrtps` was observed to drop
the message following a skipped local one when taking serialized
messages.

## Request-Response

A ROS 2 service maps to one iceoryx2 request-response service. The gateway
holds a client and a server on the ROS 2 service and a server and a client
on the iceoryx2 service, and relays requests one way and responses the
other, in both directions.

* A request is identified by the GUID of the client that sent it and the
  sequence number the client assigned. The gateway keeps the sequence
  numbers of the requests its client sent, by the link's request id, to
  route the responses back.
* `rcl` sends and takes requests and responses as the rosidl C struct of
  the type. Each endpoint holds one struct per type, allocated and
  initialized once from the introspection typesupport, and refills it per
  message.
* The rmw reports the serialized size of a struct only through the
  allocator of the serialized message it is given. The adapter serializes
  through a callback that loans the sample at that size, so the CDR bytes
  land in shared memory without a copy.
* ROS 2 has no fire-and-forget requests. The iceoryx2 setting only decides
  whether a local client may drop its pending response early. The ROS 2
  server answers regardless, and the link discards the response.

### Caveats

#### Loopback

`rcl` delivers a request to every server on a service, the gateway's own
included:

```text
gateway client ──request──▶ ROS 2 server
                       └──▶ gateway server   (skipped)
```

The gateway's server skips the requests of the gateway's own client. It
recognizes them by their participant:

* Every taken request comes with a request id, which holds a GUID of the
  client.
* The first twelve bytes of a GUID are the participant prefix. Every
  endpoint of a node has the same one, and no other node shares the
  participant of the gateway's node.
* A request whose participant prefix is the node's own is skipped. The
  whole GUID cannot be compared, since which entity of the client a
  request id names is up to the rmw.

The node learns its participant prefix from a publisher because Humble,
the earliest supported distribution, reports GIDs for publishers only.
`rmw_get_gid_for_client` exists from Iron on. The node reads the prefix
once at creation:

* It creates a publisher on `/rosout`, reads the GID and finalizes the
  publisher.
* `/rosout` is the topic every node publishes on and the gateway never
  propagates, so the probe leaves no trace on the graph.

#### No serialized form for services

`rcl` provides APIs to send and receive publish-subscribe messages over
topics in their serialized form. For request-response messages, there is
no equivalent:

| pattern | `rcl` API                                                                      | takes             |
|---------|--------------------------------------------------------------------------------|-------------------|
| topic   | `rcl_publish_serialized_message`, `rcl_take_serialized_message`                | CDR bytes         |
| service | `rcl_send_request`, `rcl_take_request_with_info` and the response counterparts | the rosidl struct |

This asymmetry was unexpected from the initial design. The following
temporary approach mitigates this but is not efficient. With the
assumption that request-response payloads are typically smaller in size
than publish-subscribe payloads, it should be sufficient for now.

The adapter expects payload bytes in the wire form on both patterns. A
request or response is therefore serialized by the translator only to be
deserialized in the adapter into the struct `rcl` takes, which the rmw
serializes again for the wire:

```text
send:  shared memory ──translator──▶ CDR ──rmw_deserialize──▶ rosidl struct ──rcl_send_*──▶ rmw ──serialize──▶ CDR on the wire
take:  shared memory ◀──translator── CDR ◀──rmw_serialize── rosidl struct ◀──rcl_take_*── rmw ◀──deserialize── CDR on the wire
```

* With the passthrough translator this is the one conversion a service
  costs.
* With the plain-struct translator a service converts twice, the topic
  once:

```text
topic,   plain struct:  iceoryx2 struct ──translator──▶ CDR ───────────────────────────────▶ rcl
service, plain struct:  iceoryx2 struct ──translator──▶ CDR ──endpoints──▶ rosidl struct ──▶ rcl
service, passthrough:                                   CDR ──endpoints──▶ rosidl struct ──▶ rcl
```

An application may store the rosidl struct itself in shared memory. The
plain-struct translator then converts it to CDR and the adapter back to
the struct, as the second row shows. Skipping both conversions requires
the endpoints to receive the struct instead of the serialized form.

#### One response per request

|                       | iceoryx2 server            | ROS 2 server |
|-----------------------|----------------------------|--------------|
| responses per request | any number                 | exactly one  |
| end of stream         | drops the request          | none         |
| abandoned request     | drops the pending response | none         |

The adapter reconciles the two:

* A response taken from a ROS 2 server is followed by the end of its
  stream on the next take.
* The first response a local server sends to a request from ROS 2 is
  sent. Any further response is dropped with a warning.
* A local server ending its stream without a response leaves the ROS 2
  client to its timeout.
* An abandoned local request has its state dropped by the gateway. A
  response still arriving for it finds no destination and is skipped.

#### Lost requests

ROS 2 services are volatile. A request sent before the sender's
participant has matched the receiver is dropped:

```text
local client ──▶ gateway client ─ ─ ✕ ─ ─▶ ROS 2 server   (not matched yet)
```

* ROS 2 clients wait for a server before calling. The link has no such
  wait, so a request sent right after the gateway opened its ROS 2 client
  may be lost, and the local client sees no response.
* A request to a ROS 2 server that never answers keeps its entry in the
  gateway, since nothing tells the gateway the request is over.

#### The GUID's element type

`rmw_request_id_t` declares the GUID as `int8_t[16]` in Humble and as
`uint8_t[16]` from Iron on. The adapter converts per element, so the same
source builds against either.
