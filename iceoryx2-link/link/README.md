# iceoryx2-link

> [!IMPORTANT]
> The link is currently a prototype and requires validation in real
> deployments. Only recommended for experimentation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

`iceoryx2` communicates through shared memory, which ends at the boundary
of the memory domain, a machine, a virtual machine, or a processor in a
system on chip with memory of its own. A link extends `iceoryx2` services
across that boundary.

## Composition

A link has two parts, a core and a backend. The core knows the local
`iceoryx2` system, the backend knows the opposing side. Services offered
locally are exported through the backend, services found through it are
mirrored as local services, so applications use them like any other.

Two kinds of backends are provided supporting different contracts, a
**tunnel** to another `iceoryx2` system and a **gateway** to another
middleware.

```text
                  local iceoryx2 system
         ┌─────────────────────────────────┐
         │           applications          │
         └────────────────┬────────────────┘
                          │ services
   ┌─ Link ───────────────┼──────────────────────────┐
   │     ┌────────────────┴────────────────┐         │
   │     │               Core              │         │
   │     │ discovery, bridges, propagation │         │
   │     └────────────────┬────────────────┘         │
   │                      │                          │
   │   ┌ ─ Backend, one of┴─ ─ ─ ─ ─ ─ ─ ─ ─ ┐       │
   │     ┌─────────────┐       ┌─────────────┐       │
   │   │ │   Tunnel    │       │   Gateway   │ │     │
   │     │             │       │  mapping,   │       │
   │   │ │             │  or   │  translator │ │     │
   │     ├─────────────┤       ├─────────────┤       │
   │   │ │   Carrier   │       │   Adapter   │ │     │
   │     └──────┬──────┘       └──────┬──────┘       │
   │   └ ─ ─ ─ ─│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ┘     │
   └────────────┼─────────────────────┼──────────────┘
                │                     │
     ───────────┼─────────────────────┼───────────  boundary
                ▼                     ▼
        another iceoryx2        another middleware
        system
```

A tunnel runs over a **carrier**. The carrier connects to the peers on the
opposing side, announces which services this side offers, and moves bytes
between the two. Services and data cross unchanged, both sides are
`iceoryx2`.

A gateway runs over an **adapter**. The adapter connects to the other
middleware, lists its endpoints (e.g. topics in ROS 2), opens them, and
moves messages in that middleware's own format. Besides the adapter a
gateway takes a mapping, which services and endpoints correspond, and a
translator, how their types and data do.
