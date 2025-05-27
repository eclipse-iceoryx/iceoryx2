FROM ubuntu:latest

RUN apt-get update && apt-get install -y \
    ca-certificates \
    procps \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -s /bin/bash testuser

RUN mkdir -p /usr/local/bin

COPY target/release/iox2-tunnel \
     target/release/iceoryx2-tunnels-end-to-end-testing-pinger \
     target/release/iceoryx2-tunnels-end-to-end-testing-ponger \
     /usr/local/bin/

RUN chmod +x /usr/local/bin/iox2-tunnel \
      /usr/local/bin/iceoryx2-tunnels-end-to-end-testing-pinger \
      /usr/local/bin/iceoryx2-tunnels-end-to-end-testing-ponger

USER testuser
WORKDIR /home/testuser

CMD ["echo", "Test container ready"]
