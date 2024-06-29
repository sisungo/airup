# Airup SDK for C API 手册
Airup SDK for C 是 Airup SDK 的一个实现，适用于 C99/C11/C23 和 C++。

🚧 **警告** 🚧: 该 SDK 正在被重写，并且旧 SDK 已从仓库删除。若要使用该 SDK，请查看旧版（v0.10.3），或等待新的 SDK 完成。

## 示例
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

这是一个简单的 Airup 客户端程序，能够启动一个服务。被启动的服务通过第一个命令行参数指定。

## 头文件
 - [`airup.h`](airup_h.md)
