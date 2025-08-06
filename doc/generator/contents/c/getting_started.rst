Getting Started
===============

.. toctree::

.. include:: ../../../../examples/c/README.md
   :parser: myst_parser.sphinx_

All examples for all languages can be found in the table in the
`iceoryx2 examples directory <https://github.com/eclipse-iceoryx/iceoryx2/tree/main/examples>`_.

The publish-subscriber example can be started with 2 terminals.

Start in terminal 1:

.. code-block:: sh

   ./target/ffi/c/build/examples/publish_subscribe/example_c_subscriber

And in terminal 2:

.. code-block:: sh

   ./target/ffi/c/build/examples/publish_subscribe/example_c_publisher

You should observe how the publisher application sends data to the subscriber application.
