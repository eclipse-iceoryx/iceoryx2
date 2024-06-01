use iceoryx2::prelude::*;
use iceoryx2_bb_log::set_log_level;

mod publisher;
mod subscriber;

pub use publisher::*;
pub use subscriber::*;

#[no_mangle]
pub extern "C" fn zero_copy_service_list() -> i32 {
    set_log_level(iceoryx2_bb_log::LogLevel::Info);

    let services = zero_copy::Service::list();

    if services.is_err() {
        return -1;
    }

    let services = services.unwrap();

    for service in services {
        println!("\n{:#?}", &service);
    }

    return 0;
}
