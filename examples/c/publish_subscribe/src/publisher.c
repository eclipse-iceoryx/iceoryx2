#include "iox2/iceoryx2.h"

#include <stdint.h>

int main(void) {
    const uint32_t NUMBER_OF_SECONDS_TO_RUN = 10;
    return run_publisher(NUMBER_OF_SECONDS_TO_RUN);
}
