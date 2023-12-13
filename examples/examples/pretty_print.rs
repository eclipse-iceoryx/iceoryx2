use std::error::Error;

use iceoryx2::service::{zero_copy, Service, service_name::ServiceName};


fn main() {
    let event_name = ServiceName::new("MyEventName").unwrap();
    let event = zero_copy::Service::new(&event_name)
        .event()
        .open_or_create().unwrap();

    println!("{}", event);   
}