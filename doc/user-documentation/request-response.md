# Request-Response

## Classes Involved In PendingResponse to ActiveRequest Stream Communication

User has send request to the server and receives a stream of responses.

```mermaid
classDiagram
    Client "1" --> "1" DataSegment: stores request payload
    Server "1" --> "1" DataSegment: stores response payload
    Client "1" --> "1..*" PendingResponse
    Server "1" --> "1..*" ActiveRequest
    ActiveRequest "1" --> "1..*" ResponseMut: loan and send
    PendingResponse "1" --> "1..*" Response: receive
    PendingResponse "1" --> "1" ZeroCopyConnection: receive Response
    ActiveRequest "1" --> "1" ZeroCopyConnection: send ResponseMut
```

## Sending Request: Client View

```mermaid
sequenceDiagram
    User->>+Client: loan
    Client-->>-User: RequestMut
    create participant RequestMut
    User->>RequestMut: write_payload
    destroy RequestMut
    User->>+RequestMut: send
    RequestMut-->>-User: PendingResponse
    create participant PendingResponse
    User->>+PendingResponse: receive
    PendingResponse-->>-User: Response
    create participant Response
    User->>Response: read_payload
    destroy PendingResponse
    User->>PendingResponse: drop
```

## Responding: Server View

```mermaid
sequenceDiagram
    User->>+Server: receive
    Server-->>-User: ActiveRequest
    create participant ActiveRequest
    User->ActiveRequest: read_payload
    User->>+ActiveRequest: loan
    ActiveRequest-->>-User: ResponseMut
    create participant ResponseMut
    User->>ResponseMut: write_payload
    destroy ResponseMut
    User->>ResponseMut: send
    destroy ActiveRequest
    User->>ActiveRequest: drop
```
