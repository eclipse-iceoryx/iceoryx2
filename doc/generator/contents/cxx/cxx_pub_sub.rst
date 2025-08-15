Publish-Subscribe
-----------------

.. toctree::

This minimal example showcases a publisher sending the number 1234 every second,
while a subscriber efficiently receives and prints the data.

This example is a simplified version of the
`publish-subscribe example <https://github.com/eclipse-iceoryx/iceoryx2/tree/main/examples/cxx/publish_subscribe>`_.
You can execute it by opening two terminals and calling:

**Terminal 1:**

.. code-block:: sh

   ./target/ff/cc/build/examples/cxx/publish_subscribe/example_cxx_publish_subscribe_subscriber

**Terminal 2:**

.. code-block:: sh

   ./target/ff/cc/build/examples/cxx/publish_subscribe/example_cxx_publish_subscribe_publisher


Publisher
^^^^^^^^^

.. code-block:: C++

   #include "iox/duration.hpp"
   #include "iox2/node.hpp"
   #include "iox2/sample_mut.hpp"
   #include "iox2/service_name.hpp"
   #include "iox2/service_type.hpp"

   #include <utility>

   constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

   auto main() -> int {
       using namespace iox2;
       auto node = NodeBuilder().create<ServiceType::Ipc>().expect("");

       auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect(""))
                          .publish_subscribe<uint64_t>()
                          .open_or_create()
                          .expect("successful service creation/opening");

       auto publisher = service.publisher_builder().create().expect("");

       while (node.wait(CYCLE_TIME).has_value()) {
           auto sample = publisher.loan_uninit().expect("acquire sample");
           auto initialized_sample = sample.write_payload(1234);
           send(std::move(initialized_sample)).expect("send successful");
       }
   }

Subscriber
^^^^^^^^^^

.. code-block:: C++

   #include <iostream>
   #include "iox/duration.hpp"
   #include "iox2/node.hpp"
   #include "iox2/service_name.hpp"
   #include "iox2/service_type.hpp"

   constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

   auto main() -> int {
       using namespace iox2;
       auto node = NodeBuilder().create<ServiceType::Ipc>().expect("");

       auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect(""))
                          .publish_subscribe<uint64_t>()
                          .open_or_create()
                          .expect("successful service creation/opening");

       auto subscriber = service.subscriber_builder().create().expect("");

       while (node.wait(CYCLE_TIME).has_value()) {
           auto sample = subscriber.receive().expect("receive succeeds");
           while (sample.has_value()) {
               std::cout << "received: " << sample->payload() << std::endl;
               sample = subscriber.receive().expect("receive succeeds");
           }
       }

       return 0;
   }
