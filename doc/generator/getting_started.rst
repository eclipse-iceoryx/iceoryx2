Getting Started!
================

Install dependencies
--------------------

Since iceoryx2 is written in Rust we need to install that first.
We recommend the `official approach <https://www.rust-lang.org/tools/install>`_.

.. code-block:: sh

   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Then C/C++ compiler and library dependencies must be installed.

ArchLinux
^^^^^^^^^

.. code-block:: sh

   sudo ./internal/scripts/install_dependencies_archlinux.sh

Ubuntu
^^^^^^

.. code-block:: sh

   sudo ./internal/scripts/install_dependencies_ubuntu.sh

Build
-----

In the repository root folder, execute the following steps.

.. code-block:: sh

   cmake -S . -B target/ffi/build -DBUILD_EXAMPLES=ON
   cmake --build target/ffi/build

Examples
--------

.. toctree::
   :maxdepth: 1

  getting_started_c

  getting_started_cxx
