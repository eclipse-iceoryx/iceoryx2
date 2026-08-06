FROM ubuntu:latest

ARG ENTRYPOINT_BIN
ARG BACKEND_BIN
ARG PINGER_BIN
ARG PONGER_BIN

RUN apt-get update && apt-get install -y \
    ca-certificates \
    procps \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -s /bin/bash testuser

RUN mkdir -p /usr/local/bin

COPY ${ENTRYPOINT_BIN} ${BACKEND_BIN} ${PINGER_BIN} ${PONGER_BIN} /usr/local/bin/

RUN chmod +x /usr/local/bin/*

USER testuser
WORKDIR /home/testuser

CMD ["echo", "Test container ready"]
