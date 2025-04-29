# Using iceoryx2 in A Docker Environment

## Introduction

Let's assume we have a setup where the publisher and the subscriber are running
in their own docker containers. We use the
[publish_subscribe](../publish_subscribe) as base.

Our setup looks like:

```text
                    +------------------------------+
                    | host                         |
                    |                              |
                    | publish_subscribe_subscriber |
                    +------------------------------+
                       ^
                  send | data
                       |
 +-----------------------------+         +------------------------------+
 | docker 1                    |  send   | docker 2                     |
 |                             |-------->|                              |
 | publish_subscribe_publisher |  data   | publish_subscribe_subscriber |
 +-----------------------------+         +------------------------------+
```

On our host we would like to run a subscriber, on docker container 1 a publisher
that can send data to the subscriber in docker container 2 and to the host.

## Requirements

iceoryx2 discovers services by parsing the service toml files in the
`/tmp/iceoryx2` directory and communicates via shared memory that is located in
`/dev/shm`. If both directories are available in every docker container and are
shared with the host, iceoryx2 can establish a connection between them.

## How to Run

We start three separate terminals. We use `archlinux:latest` in this example
but you are free to choose any other linux distribution.

We start by building the example:

```sh
cargo build --example publish_subscribe_publisher
cargo build --example publish_subscribe_subscriber
```

Create the directory `/tmp/iceoryx2` so that it can be mounted into the docker
container. iceoryx2 creates it automatically but in our case we start the
container before we start iceoryx2, therefore we have to create it manually.

```sh
mkdir /tmp/iceoryx2
```

Now open the terminals.

### Terminal 1 (docker 1)

```sh
docker run --mount type=bind,source="/dev/shm",target=/dev/shm --mount \
    type=bind,source=/home/$USER$/iceoryx2,target=/iceoryx2 --mount \
    type=bind,source=/tmp/iceoryx2,target=/tmp/iceoryx2 -it archlinux:latest

cd /iceoryx2
./target/debug/examples/publish_subscribe_publisher
```

### Terminal 2 (docker 2)

```console
docker run --mount type=bind,source="/dev/shm",target=/dev/shm --mount \
    type=bind,source=/home/$USER$/iceoryx2,target=/iceoryx2 --mount \
    type=bind,source=/tmp/iceoryx2,target=/tmp/iceoryx2 -it archlinux:latest

cd /iceoryx2
./target/debug/examples/publish_subscribe_subscriber
```

### Terminal 3 (host)

Docker is mostly started as root and therefore all the shared memory segments
and service files are created under the root user. We use `sudo` to be able to
subscribe to the docker services.

```console
cd iceoryx2
sudo ./target/debug/examples/publish_subscribe_subscriber
```

## docker-compose Example

We can also use `docker-compose` to start our test setup. Our example is coming
with a configuration file `docker-compose.yml` which can be used from the
iceoryx root path with the following command:

```console
mkdir /tmp/iceoryx2
docker-compose -f examples/rust/docker/docker-compose.yml --project-directory . up
```

We can again open a new terminal and start an additional publisher or subscriber
with root privileges and connect to the docker containers.

```console
cd iceoryx2
sudo ./target/debug/examples/publish_subscribe_publisher
```
