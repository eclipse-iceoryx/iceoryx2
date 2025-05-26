# Lightweight runtime container for pre-built binaries
FROM ubuntu:22.04

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -s /bin/bash testuser

# Create directory for binaries
RUN mkdir -p /usr/local/bin

# Binaries (combined into a single layer with chmod in the same layer)
COPY target/release/iox2 \
     target/release/iox2-tunnel \
     target/release/iceoryx2-tunnels-end-to-end-testing-pinger \
     target/release/iceoryx2-tunnels-end-to-end-testing-ponger \
     /usr/local/bin/

# Make binaries executable
RUN chmod +x /usr/local/bin/iox2 \
      /usr/local/bin/iox2-tunnel \
      /usr/local/bin/iceoryx2-tunnels-end-to-end-testing-pinger \
      /usr/local/bin/iceoryx2-tunnels-end-to-end-testing-ponger

# Switch to non-root user
USER testuser
WORKDIR /home/testuser

# Default command (overridden in docker-compose)
CMD ["echo", "Test container ready"]
