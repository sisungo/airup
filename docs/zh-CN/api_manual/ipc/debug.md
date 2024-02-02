# `debug` 模块
`debug` 模块内提供了用于调试 Airup 的方法。

## `debug.disconnect` 方法
 - **名称**：`debug.disconnect`
 - **参数**：无
 - **返回值**：永不返回
 - **描述**：断开当前 IPC 连接。

## `debug.exit` 方法
 - **名称**：`debug.exit`
 - **参数**：无
 - **返回值**：`null`
 - **描述**：让 `airupd` 守护进程退出。

## `debug.echo_raw` 方法
 - **名称**：`debug.echo_raw`
 - **参数**：`Response` 对象
 - **返回值**：参数的内容
 - **描述**：返回参数中提供的 `Response` 对象。

## `debug.reload_image` 方法
 - **名称**：`debug.reload_image`
 - **参数**：无
 - **返回值**：`null`
 - **描述**：让 `airupd` 守护进程重新加载其进程映像。