#####################
iceoryx2: User Manual
#####################

.. image:: https://user-images.githubusercontent.com/8661268/114321508-64a6b000-9b1b-11eb-95ef-b84c91387cff.png
   :width: 500
   :alt: iceoryx logo

Welcome to iceoryx2, the efficient, and ultra-low latency inter-process communication
middleware. This library is designed to provide you with fast and reliable
zero-copy and lock-free inter-process communication mechanisms.

So if you want to communicate efficiently between multiple processes or applications
iceoryx2 is for you. With iceoryx2, you can:

* Send huge amounts of data using a publish/subscribe, request/response (planned),
  pipeline (planned) or blackboard pattern (planned),
  making it ideal for scenarios where large datasets need to be shared.
* Exchange signals through events, enabling quick and reliable signaling
  between processes.

iceoryx2 is based on a service-oriented architecture (SOA) and facilitates
seamless inter-process communication (IPC).

Choose your language:

.. grid:: 4
    :margin: 4 4 4 4

    .. grid-item::
        .. card:: C
            :link: contents/c/index.html

    .. grid-item::
        .. card:: C++
            :link: contents/cxx/index.html

    .. grid-item::
        .. card:: Python
            :link: contents/python/index.html

    .. grid-item::
        .. card:: Rust
            :link: contents/rust/index.html


.. toctree::
   :maxdepth: 1
   :hidden:

   contents/c/index
   contents/cxx/index
   contents/python/index
   contents/rust/index
