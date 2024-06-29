#include <airup/rpc.h>
#include <airup/error.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>

static struct airup_error io_error(void) {
    union airup_error_payload payload;
    payload.sys_errno = errno;
    struct airup_error error = { AIRUP_IO_ERROR, payload };
    return error;
}

int airup_connect(airup_conn_t *obj, const char *path) {
    airup_conn_t conn = { -1 };
    struct sockaddr_un addr;
    socklen_t addrlen = sizeof(addr);
    if ((conn.sockfd = socket(AF_UNIX, SOCK_STREAM, 0)) < 0) {
        airup_set_error(io_error());
        return -1;
    }
    memset((void*)&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, path, sizeof(addr.sun_path) - 1);
    if (connect(conn.sockfd, (struct sockaddr*)&addr, addrlen) < 0) {
        airup_set_error(io_error());
        return -1;
    }
    *obj = conn;
    return 0;
}

void airup_disconnect(airup_conn_t obj) {
    close(obj.sockfd);
}
