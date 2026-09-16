FROM ubuntu:latest

ARG ENTRYPOINT_BIN
ARG LINK_BIN
ARG TUNNEL_BIN
ARG PINGER_BIN
ARG PONGER_BIN
ARG CONFIG_FILE

RUN apt-get update && apt-get install -y \
    ca-certificates \
    procps \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -s /bin/bash testuser

RUN mkdir -p /usr/local/bin

COPY ${ENTRYPOINT_BIN} ${LINK_BIN} ${TUNNEL_BIN} ${PINGER_BIN} ${PONGER_BIN} /usr/local/bin/
COPY ${CONFIG_FILE} /home/testuser/config/iceoryx2.toml

RUN chmod +x /usr/local/bin/*

USER testuser
WORKDIR /home/testuser

CMD ["echo", "Test container ready"]
