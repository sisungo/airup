# 头文件：`airup.h`

## 结构体：`airup_error`
```c
#define AIRUP_EIO 16
#define AIRUP_EAPI 32

struct airup_error {
    uint32_t code;
    const char *message;
    const void *payload;
};
```

**描述**：表示调用 Airup SDK 函数时发生的错误。

**字段** *`code`*：表示错误的类型。

**字段** *`message`*：UTF-8 编码的字符串，以纯文本描述该错误的信息。

**字段** *`payload`*：该错误的附加信息。其类型取决于 `code` 字段的值。

**宏** *`AIRUP_EIO`*：错误代码，表示该错误由操作系统 IO 失败导致。

**宏** *`AIRUP_EAPI`*：错误代码，表示该错误由从 Airupd 服务器返回的 API 错误导致。当 `code` 字段被设置为 `AIRUP_EAPI` 时，`payload` 字段的
类型将为 `struct airup_api_error`。

## 结构体：`airup_api_error`
```c
struct airup_api_error {
    const char *code;
    const char *message;
    const char *json;
};
```

**描述**：表示从 Airupd 服务器返回的 API 错误。

**字段** *`code`*：UTF-8 编码的字符串，表示错误代码。

**字段** *`message`*：UTF-8 编码的字符串，以纯文本描述该错误的信息。

**字段** *`json`*：从 Airupd 服务器接收到的原始 JSON 字符串，以 UTF-8 编码。

## 函数：`airup_last_error`
```c
struct airup_error airup_last_error(void);
```

**描述**：返回上一次调用 Airup SDK 函数出错时发生的错误。该函数是线程安全的，因为 Airup 错误属于线程本地存储。

## 函数：`airup_connect`
```c
airup_connection *airup_connect(const char *path);
```

**描述**：尝试以 Airup 的 IPC 协议连接到指定路径 `path` 上的 Airup IPC 端口。如果成功，返回指向打开的连接的指针。如果失败，返回 `NULL`，并设置
当前线程的 Airup 错误。

## 函数: `airup_disconnect`
```c
void airup_disconnect(airup_connection *connection);
```

**描述**：关闭连接 `connection`。调用该方法后 `connection` 将被释放并不再可用。

## 函数：`airup_default_path`
```c
const char *airup_default_path(void);
```

**描述**：获取默认的 Airup IPC 端口路径。如果设置了 `AIRUP_SOCK` 环境变量，返回该环境变量的值，否则返回根据构建该 SDK 时
的 `build_manifest.json` 中计算出的路径。

## 函数：`airup_build_manifest`
```c
const char *airup_build_manifest(void);
```

**描述**：获取此 SDK 的构建清单的 JSON 字符串表示，或称为此 Airup SDK 的编译时 `build_manifest.json` 的内容。

## 函数：`airup_start_service`
```c
int airup_start_service(airup_connection *connection, const char *name);
```

**描述**：在连接 `connection` 上调用 `system.start_service` 方法并传递 `name` 作为参数。如果成功，返回 `0`。如果失败，返回 `-1` 并设置当前
线程的 Airup 错误。

## 函数：`airup_stop_service`
```c
int airup_stop_service(airup_connection *connection, const char *name);
```

**描述**：在连接 `connection` 上调用 `system.stop_service` 方法并传递 `name` 作为参数。如果成功，返回 `0`。如果失败，返回 `-1` 并设置当前
线程的 Airup 错误。

## 函数：`airup_trigger_event`
```c
int airup_trigger_event(airup_connection *connection, const char *event);
```

**描述**：在连接 `connection` 上调用 `system.trigger_event` 方法并传递 `event` 作为参数。如果成功，返回 `0`。如果失败，返回 `-1` 并设置当前
线程的 Airup 错误。
