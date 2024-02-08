# Airup SDK for C manual
Airup SDK for C is an implementation of the Airup SDK. It is for C99/C11/C23 and C++ prior than C++ 23.

## Example
```c
#include <airup.h>
#include <stdio.h>

int main(int argc, char *argv[]) {
    char *path = airup_default_path();
    airup_connection *conn = airup_connect(path);
    if (conn == NULL) {
        printf("error: failed to connect to airup daemon: %s\n", airup_last_error().message);
        return 1;
    }
    if (argc > 1) {
        int status = airup_start_service(conn, argv[1]);
        if (status == -1) {
            printf("error: failed to start service %s: %s\n", argv[1], airup_last_error().message);
            airup_disconnect(conn);
            return 1;
        }
    } else {
        printf("error: no service specified to start\n");
        airup_disconnect(conn);
        return 1;
    }
}
```

This is a simple Airup client program which starts a service read from its first command-line argument.

## Header Files
 - [`airup.h`](airup_h.md)
 