# `info` 模块
`info` 模块提供了用于查询有关 Airup 和系统的信息的方法。

## `info.version` 方法
 - **名称**：`info.version`
 - **参数**：无
 - **返回值**：`string`
 - **描述**：查询 `airupd` 的版本。

 ## `info.build_manifest` 方法
 - **名称**：`info.build_manifest`
 - **参数**：无
 - **返回值**：`BuildManifest` 对象
 - **描述**：返回构建 `airupd` 时所嵌入的 `build_manifest.json`。