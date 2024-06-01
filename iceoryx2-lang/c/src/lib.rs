use iceoryx2::prelude::*;

#[no_mangle]
pub extern "C" fn zero_copy_service_list() -> i32 {
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
