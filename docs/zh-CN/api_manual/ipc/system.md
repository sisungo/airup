# `system` 模块
`system` 模块提供了用于管理系统的方法。

## `system.refresh` 方法
- 名称：`system.refresh`
- 参数：无
- 需要的权限：`refresh`
- 返回值：`null`
- 描述：刷新 `airupd` 缓存的系统资源和内部状态。

## `system.query_service` 方法
- 名称：`system.query_service`
- 参数：`字符串（要查询的服务名称）`（可选）
- 需要的权限：`query_services`
- 返回值：`QueryResult` 对象 *或* `字符串数组`
- 描述：若参数为 `null`，返回包含所有已加载的服务名称的列表。如果不，则返回该服务查询到的状态。

## `system.start_service` 方法
- 名称：`system.start_service`
- 参数：`字符串（要操作的服务名称）`
- 需要的权限：`manage_services`
- 返回值：`null`
- 描述：启动指定的服务。

## `system.stop_service` 方法
- 名称：`system.stop_service`
- 参数：`字符串（要操作的服务名称）`
- 需要的权限：`manage_services`
- 返回值：`null`
- 描述：停止指定的服务。