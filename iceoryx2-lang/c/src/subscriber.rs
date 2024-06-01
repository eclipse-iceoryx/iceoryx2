use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

#[no_mangle]
pub extern "C" fn run_subscriber(seconds: u32) -> i32 {
    let service_name = ServiceName::new("Hello/from/C");

    if service_name.is_err() {
        return -1;
    }

    let service_name = service_name.unwrap();

    let service = zero_copy::Service::new(&service_name)
        .publish_subscribe::<u64>()
        .open_or_create();

    if service.is_err() {
        return -1;
    }

    let service = service.unwrap();

    let subscriber = service.subscriber().create();

    if subscriber.is_err() {
        return -1;
    }

    let subscriber = subscriber.unwrap();

    let mut remaining_seconds = seconds;

    while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
        loop {
            match subscriber.receive() {
                Ok(Some(sample)) => println!("received: {:?}", *sample),
                Ok(None) => break,
                Err(_) => return -1,
            }
        }

        remaining_seconds = remaining_seconds.saturating_sub(1);
        if remaining_seconds == 0 {
            break;
        }
    }

    println!("exit");

    return 0;
}
