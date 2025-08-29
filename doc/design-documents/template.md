# Design Document Template

A design document is a **living document** during the feature implementation
phase. It’s expected to evolve: not every detail is known at the beginning,
and the implementation may diverge from the initial plan. Updates should be
included in the pull request (PR) where the feature is implemented, so reviewers
can see both the rationale and the changes in one place.

This template is a **recommendation**, not a rigid rule. Use only the sections
that make sense for your case.

Graphics are encouraged:

* Include with `![Title](graphic.svg){width=60%}`
* Create with **draw\.io** (export as SVG)
* Inline **mermaid diagrams** are also allowed

## Terminology

Define key terms used in this document. Keep explanations concise.

* **Some Term** – One-sentence explanation.
* **Publish-Subscribe Messaging Pattern** – Uni-directional communication where
  a publisher (sender) sends a stream of data to a subscriber (receiver).

## Overview

A high-level pitch of the feature:

* What new capability does it provide?
* What problems does it solve that we couldn’t solve before?
* Why is it worth building?

## Requirements

State **behavioral requirements**, not implementation details.

Example:

* **R1: Low-Latency Communication** – The communication shall be
  memory-efficient and have low latency. (This implies zero-copy under the hood
  but doesn’t prescribe it.)

## Use Cases

Describe how the feature is valuable in real scenarios.

### Use-Case 1: Function Runtime Tracing

* **As a** developer optimizing iceoryx2 usage
* **I want** to trace the runtime of individual iceoryx2 functions
* **So that** I can optimize performance

* **Given** a running iceoryx2 system
* **When** I activate tracing
* **Then** I record the runtime of all iceoryx2 functions

## Usage

Explain the feature from a user’s perspective. Include example code snippets.
Use subsections for each subfeature.

### Example: Print Hello World

```rust
println!("hello world from my feature");
```

## Implementation

Outline the overall approach. Provide subsections for major subfeatures or
milestones. Visuals (class diagrams, sequence diagrams, etc.) are strongly
encouraged.

## Certification & Safety-Critical Usage

Address whether and how the feature can be used in safety-critical or certified
environments. Consider:

* Applicable standards (e.g., ASIL-D, ISO 26262)
* Support for **zero-trust deployments**: can rogue processes break it?
* Evidence for claims (e.g., subscribers cannot corrupt publisher-owned
  read-only memory)
* Real-time suitability: any blocking calls, background threads, or indeterminism?
* If unsuitable for zero-trust or real-time use, how do we prevent accidental misuse?

## Milestones

List development phases and results.

### Milestone 1 – Implement A

* Planned classes, modules, details

**Results:**

* What the user will see or gain from this milestone
