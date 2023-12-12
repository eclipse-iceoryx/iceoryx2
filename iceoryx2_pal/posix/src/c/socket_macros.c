#ifdef _WIN64
#include <WinSock2.h>
#include <Windows.h>
#include <MSWSock.h>
#include <io.h>



#else
#include <sys/select.h>
#include <sys/socket.h>

size_t iceoryx2_cmsg_space(const size_t len) { return CMSG_SPACE(len); }

struct cmsghdr* iceoryx2_cmsg_firsthdr(const struct msghdr* hdr) {
    return CMSG_FIRSTHDR(hdr);
}

struct cmsghdr* iceoryx2_cmsg_nxthdr(struct msghdr* hdr, struct cmsghdr* sub) {
    return CMSG_NXTHDR(hdr, sub);
}

size_t iceoryx2_cmsg_len(const size_t len) { return CMSG_LEN(len); }

unsigned char* iceoryx2_cmsg_data(struct cmsghdr* cmsg) {
    return CMSG_DATA(cmsg);
}

void iceoryx2_fd_clr(const int fd, fd_set* set) { FD_CLR(fd, set); }

int iceoryx2_fd_isset(const int fd, const fd_set* set) {
    return FD_ISSET(fd, set);
}

void iceoryx2_fd_set(const int fd, fd_set* set) { FD_SET(fd, set); }

void iceoryx2_fd_zero(fd_set* set) { FD_ZERO(set); }

#endif
