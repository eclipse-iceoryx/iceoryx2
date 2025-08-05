Event
-----

.. toctree::

This minimal example showcases how push-notifications can be realized by using services
with event messaging pattern between two processes. The listener hereby waits for a
notification from the notifier.

This example is a simplified version of the
`event example <https://github.com/eclipse-iceoryx/iceoryx2/tree/main/examples/cxx/event>`_.
You can execute it by opening two terminals and calling:

**Terminal 1:**

.. code-block:: sh

   ./target/ff/cc/build/examples/cxx/event/example_cxx_event_listener

**Terminal 2:**

.. code-block:: sh

   ./target/ff/cc/build/examples/cxx/event/example_cxx_event_notifier


Listener
^^^^^^^^

.. code-block:: C++

   #include "iox/duration.hpp"
   #include "iox2/node.hpp"
   #include "iox2/service_name.hpp"
   #include "iox2/service_type.hpp"

   #include <iostream>

   constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

   auto main() -> int {
       using namespace iox2;
       auto node = NodeBuilder().create<ServiceType::Ipc>().expect("");

       auto service = node.service_builder(ServiceName::create("MyEventName").expect(""))
                          .event()
                          .open_or_create()
                          .expect("successful service creation/opening");

       auto listener = service.listener_builder().create().expect("");

       while (node.wait(iox::units::Duration::zero()).has_value()) {
           listener.timed_wait_one(CYCLE_TIME).and_then([](auto maybe_event_id) {
               maybe_event_id.and_then(
                   [](auto event_id) {
                        std::cout << "event was triggered with id: " << event_id << std::endl;
                    });
           });
       }

       return 0;
   }

Notifier
^^^^^^^^

.. code-block:: C++

   #include "iox/duration.hpp"
   #include "iox2/event_id.hpp"
   #include "iox2/node.hpp"
   #include "iox2/service_name.hpp"
   #include "iox2/service_type.hpp"

   #include <iostream>

   constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

   auto main() -> int {
       using namespace iox2;
       auto node = NodeBuilder().create<ServiceType::Ipc>().expect("");

       auto service = node.service_builder(ServiceName::create("MyEventName").expect(""))
                          .event()
                          .open_or_create()
                          .expect("successful service creation/opening");
       auto max_event_id = service.static_config().event_id_max_value();

       auto notifier = service.notifier_builder().create().expect("");

       while (node.wait(CYCLE_TIME).has_value()) {
           const auto event_id = EventId(1234);
           notifier.notify_with_custom_event_id(event_id).expect("notification");

           std::cout << "Trigger event with id " << event_id << "..." << std::endl;
       }

       return 0;
   }
