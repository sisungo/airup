# Header File: `airup.h`

## Struct: `struct airup_error`
```c
#define AIRUP_EIO 16
#define AIRUP_EAPI 32

struct airup_error {
    uint32_t code;
    const char *message;
    const void *payload;
};
```

**Description**: Representation of an error caused by call to Airup SDK functions.

**Field** *`code`*: Represents to type of the error.

**Field** *`message`*: UTF-8 encoded string which describes the error in plain text.

**Field** *`payload`*: Payload information attached to the error. Its type depends on value of field `code`.

**Macro** *`AIRUP_EIO`*: An error code, which indicates the error is caused by an operating system IO failure.

**Macro** *`AIRUP_EAPI`*: An error code, which indicates the error is an API error returned from the Airupd server. When
the field `code` is set to `AIRUP_EAPI`, type of field `payload` is `struct airup_api_error`.

## Struct: `struct airup_api_error`
```c
struct airup_api_error {
    const char *code;
    const char *message;
    const char *json;
};
```

**Description**: Representation of an API error returned from Airupd server.

**Field** *`code`*: UTF-8 encoded string which represents to the error code.

**Field** *`message`*: UTF-8 encoded string which describes the error in plain text.

**Field** *`json`*: Contains raw UTF-8 encoded JSON string which is received from the Airupd server.

## Function: `airup_last_error`
```c
struct airup_error airup_last_error(void);
```

**Description**: Returns the error occurred by last call to an Airup SDK function. This is thread-safe, since Airup errors
are in thread-local storage.

## Function: `airup_connect`
```c
airup_connection *airup_connect(const char *path);
```

**Description**: Attempts to connect to IPC port on specified `path` in Airup's IPC protocol. On success, a pointer to the
connection opened is returned. On failure, NULL is returned and current thread's Airup error is set.

## Function: `airup_disconnect`
```c
void airup_disconnect(airup_connection *connection);
```

**Description**: Closes the connection `connection`. After calling this method `connection` is released and is no longer
available.

## Function: `airup_default_path`
```c
const char *airup_default_path(void);
```

**Description**：Returns default path to Airup's IPC port. If environment variable `AIRUP_SOCK` is present, its value is
returned. Otherwise a value calculated from `build_manifest.json` provided at compile-time of this SDK is returned.

## Function: `airup_build_manifest`
```c
const char *airup_build_manifest(void);
```

**Description**：Returns JSON string representation of the SDK's built-in build manifest, a.k.a content of compile-time
`build_manifest.json` of the SDK.

## Function: `airup_start_service`
```c
int airup_start_service(airup_connection *connection, const char *name);
```

**Description**: Invokes `system.start_service` method on connection `connection` with parameter `name`. On success,
returns `0`. On failure, returns `-1` and current thread's Airup error is set.

## Function: `airup_stop_service`
```c
int airup_stop_service(airup_connection *connection, const char *name);
```

**Description**: Invokes `system.stop_service` method on connection `connection` with parameter `name`. On success,
returns `0`. On failure, returns `-1` and current thread's Airup error is set.

## Function: `airup_trigger_event`
```c
int airup_trigger_event(airup_connection *connection, const char *event);
```

**Description**: Invokes `system.trigger_event` method on connection `connection` with parameter `event`. On success,
returns `0`. On failure, returns `-1` and current thread's Airup error is set.
