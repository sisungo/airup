# `system` 模块
`system` 模块提供了用于管理系统的方法。

## `system.refresh` 方法
- **名称**：`system.refresh`
- **参数**：无
- **返回值**：`null`
- **描述**：刷新 `airupd` 的一些内部状态。

## `system.gc` 方法
- **名称**：`system.gc`
- **参数**：无
- **返回值**：`null`
- **描述**：释放 `airupd` 缓存的系统资源。

## `system.query_service` 方法
- **名称**：`system.query_service`
- **参数**：`字符串（要查询的服务名称）`
- **返回值**：`QueryService` 对象
- **描述**：返回查询到的有关该服务的信息。

## `system.query_system` 方法
- **名称**：`system.query_system`
- **参数**：无
- **返回值**：`QuerySystem` 对象
- **描述**：返回查询到的关于整个系统的宏观信息。

## `system.list_services` 方法
- **名称**：`system.list_services`
- **参数**：无
- **返回值**：`string` 数组
- **描述**：返回系统中已安装的所有服务的名称的列表。

## `system.start_service` 方法
- **名称**：`system.start_service`
- **参数**：`字符串（要操作的服务名称）`
- **返回值**：`null`
- **描述**：启动指定的服务。

## `system.stop_service` 方法
- **名称**：`system.stop_service`
- **参数**：`字符串（要操作的服务名称）`
- **返回值**：`null`
- **描述**：停止指定的服务。

## `system.cache_service` 方法
- **名称**：`system.cache_service`
- **参数**：`字符串（要操作的服务名称）`
- **返回值**：`null`
- **描述**：缓存指定的服务。

## `system.uncache_service` 方法
- **名称**：`system.uncache_service`
- **参数**：`字符串（要操作的服务名称）`
- **返回值**：`null`
- **描述**：取消缓存指定的服务。

## `system.sideload_service` 方法
- **名称**：`system.sideload_service`
- **参数**：`字符串（服务名称）`, `Service` 对象
- **返回值**：`null`
- **描述**：以指定名称侧载给出的服务。

## `system.unsideload_service` 方法
- **名称**：`system.unsideload_service`
- **参数**：`字符串（要操作的服务名称）`
- **返回值**：`null`
- **描述**：从侧载存储删除指定服务。

## `system.kill_service` 方法
- **名称**：`system.kill_service`
- **参数**：`字符串（要操作的服务名称）`
- **返回值**：`null`
- **描述**：强制停止指定的服务。

## `system.reload_service` 方法
- **名称**：`system.reload_service`
- **参数**：`字符串（要操作的服务名称）`
- **返回值**：`null`
- **描述**：通知指定的服务重新加载。