# In-Progress

## Prerequesites

* Install flatbuffers:
  ```sh
  # archlinux
  pacman -S flatbuffers

  # debian/ubuntu
  apt install flatbuffers
  ```

## Usage

```sh
export IOX2_FLATBUFFER_SCHEMA_PATH=${pwd}/examples/rust/flatbuffer_publish_subscribe

flatc --rust unbounded_data.fbs
```
