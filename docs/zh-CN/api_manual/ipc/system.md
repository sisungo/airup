# `system` 模块

`system` 模块提供了用于管理系统的方法。

## `Event` 对象

**名称**：`Event`

**字段**：
 - `id [string]`：该事件的 ID。
 - `payload [string]`：该事件的负载数据。

## `system.refresh` 方法

**名称**：`system.refresh`

**参数**：无

**返回值**：`null`

**描述**：刷新 `airupd` 的一些内部状态。

## `system.gc` 方法

**名称**：`system.gc`

**参数**：无

**返回值**：`null`

**描述**：释放 `airupd` 缓存的系统资源。

## `system.query_service` 方法

**名称**：`system.query_service`

**参数**：`字符串（要查询的服务名称）`

**返回值**：`QueryService` 对象

**描述**：返回查询到的有关该服务的信息。

## `system.query_system` 方法

**名称**：`system.query_system`

**参数**：无

**返回值**：`QuerySystem` 对象

**描述**：返回查询到的关于整个系统的宏观信息。

## `system.list_services` 方法

**名称**：`system.list_services`

**参数**：无

**返回值**：`string` 数组

**描述**：返回系统中已安装的所有服务的名称的列表。

## `system.start_service` 方法

**名称**：`system.start_service`

**参数**：`字符串（要操作的服务名称）`

**返回值**：`null`

**描述**：启动指定的服务。

## `system.stop_service` 方法

**名称**：`system.stop_service`

**参数**：`字符串（要操作的服务名称）`

**返回值**：`null`

**描述**：停止指定的服务。

## `system.cache_service` 方法

**名称**：`system.cache_service`

**参数**：`字符串（要操作的服务名称）`

**返回值**：`null`

**描述**：缓存指定的服务。

## `system.uncache_service` 方法

**名称**：`system.uncache_service`

**参数**：`字符串（要操作的服务名称）`

**返回值**：`null`

**描述**：取消缓存指定的服务。

## `system.sideload_service` 方法

**名称**：`system.sideload_service`

**参数**：`字符串（服务名称）`, `Service` 对象, `bool`

**返回值**：`null`

**描述**：以指定名称侧载给出的服务。如果第三个参数为 `true`，如果指定名称的侧载服务已经存在，将会覆盖该槽位而不是返回错误。

## `system.unsideload_service` 方法

**名称**：`system.unsideload_service`

**参数**：`字符串（要操作的服务名称）`

**返回值**：`null`

**描述**：从侧载存储删除指定服务。

## `system.kill_service` 方法

**名称**：`system.kill_service`

**参数**：`字符串（要操作的服务名称）`

**返回值**：`null`

**描述**：强制停止指定的服务。

## `system.reload_service` 方法

**名称**：`system.reload_service`

**参数**：`字符串（要操作的服务名称）`

**返回值**：`null`

**描述**：通知指定的服务重新加载。

## `system.trigger_event` 方法

**名称**：`system.trigger_event`

**参数**：`Event` 对象

**返回值**：`null`

**描述**：触发指定事件。

## `system.load_extension` 方法

**名称**：`system.load_extension`

**参数**：`字符串 (扩展名称)`, `字符串 (扩展套接字路径)`

**返回值**：`null`

**描述**：加载一个 Airup 扩展。

## `system.unload_extension` 方法

**名称**：`system.unload_extension`

**参数**：`字符串 (扩展名称)`

**返回值**：`null`

**描述**：卸载一个 Airup 扩展。

**可能的错误**：

`NOT_FOUND`：指定的扩展还未被安装。
